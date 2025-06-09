use ndarray::{array, Array2};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::types::PyAny;
use pyo3::types::PyList;
use spl_pod::solana_pubkey::Pubkey;
use anyhow::Result;
use opentelemetry::global;
use opentelemetry::trace::Tracer;
use qtrade_shared_types::ArbitrageResult;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tokio::sync::mpsc;
use tracing::{error, info};
use tokio::sync::Mutex;
use lazy_static::lazy_static;
use qtrade_relayer;
use async_trait::async_trait;

// Add our DEX quoting module
pub mod dex;

// Define placeholder structs for different pool data types
// These would be replaced with actual data structures from your project

/// Placeholder for Orca Whirlpool data structure
#[derive(Debug)]
struct OrcaWhirlpoolData {
    pub sqrt_price: u128,
    pub tick_current_index: i32,
    pub liquidity: u128,
    pub fee_rate: u16,
    pub tick_spacing: u16,
    // Add other fields as needed
}

/// Placeholder for Raydium CPMM data structure
#[derive(Debug)]
struct RaydiumCpmmData {
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    pub fee_rate: u16,
    // Add other fields as needed
}

/// Placeholder for Raydium CLMM data structure
#[derive(Debug)]
struct RaydiumClmmData {
    pub sqrt_price: u128,
    pub tick_current_index: i32,
    pub liquidity: u128,
    pub fee_rate: u16,
    pub tick_spacing: u16,
    // Add other fields as needed
}


const ROUTER: &str = "router";
const CHECK_INTERVAL: Duration = Duration::from_secs(60);
const QTRADE_ROUTER_TRACER_NAME: &str = "qtrade_router";

// Global channel for passing arbitrage results from router to relayer
lazy_static! {
    pub static ref ARBITRAGE_SENDER: Mutex<mpsc::Sender<ArbitrageResult>> = {
        let (tx, rx) = mpsc::channel::<ArbitrageResult>(100);

        // Store receiver somewhere accessible to relayer
        qtrade_relayer::init_arbitrage_receiver(rx);

        // Wrap the sender in a Mutex for thread-safe access
        Mutex::new(tx)
    };
}

// Use the PoolCache trait and PoolEntry type from qtrade-shared-types
pub use qtrade_shared_types::PoolCache;
pub use qtrade_shared_types::PoolEntry;

/// Periodically performs convex optimization tasks.
///
/// This function sets up a timer to periodically:
/// - Read the pool reserves cache
/// - Read the oracle cache
/// - Call appropriate DEX module APIs for quotes based on reserves
/// - Determine arbitrage opportunities
/// - Output results to the relayer queue
pub async fn run_router<T: PoolCache + 'static>(pool_cache: Arc<T>) -> Result<()> {
    let tracer = global::tracer(QTRADE_ROUTER_TRACER_NAME);
    // Clone the pool_cache Arc once outside the loop to avoid lifetime issues
    let pool_cache_ref = Arc::clone(&pool_cache);

    loop {
        let span_name = format!("{}::run_router", ROUTER);
        // Clone another reference to the pool_cache for this iteration
        let pool_cache_iteration = Arc::clone(&pool_cache_ref);

        let result: Result<(), anyhow::Error> = tracer.in_span(span_name, move |_cx| async move {
            // Read pool reserves cache
            info!("Reading pool reserves cache...");

            // Get entries from the PoolCache instance
            let pool_entries = pool_cache_iteration.get_all_entries_as_slice().await;
            info!("Retrieved {} pool entries from cache", pool_entries.len());

            // Call appropriate DEX module APIs for quotes based on reserves
            info!("Calling DEX module APIs for quotes based on reserves...");
            // Get quotes from DEXes using our new module
            let quotes = get_dex_quotes(&pool_entries)?;
            info!("Retrieved {} quotes from DEXes", quotes.len());

            // Determine arbitrage opportunities
            info!("Determining arbitrage opportunities...");

            // We need to use the original entries directly rather than trying to clone them
            // The issue is that Box<dyn Any + Send + Sync> doesn't implement Clone
            // Since the solve function takes a reference, we can pass references to the original entries
            let router_entries = pool_entries;

            match solve(&router_entries) {
                Ok(result) => {
                    info!("Arbitrage opportunities determined successfully with status: {}", result.status);

                    // Output results to relayer queue
                    info!("Sending arbitrage results to relayer queue...");

                    // Acquire the mutex lock to access the sender
                    let sender = ARBITRAGE_SENDER.lock().await;
                    if let Err(e) = sender.send(result).await {
                        error!("Failed to send arbitrage result to relayer: {:?}", e);
                    } else {
                        info!("Successfully sent arbitrage result to relayer queue");
                    }
                },
                Err(e) => {
                    error!("Failed to determine arbitrage opportunities: {:?}", e);
                }
            }

            // Output completion message
            info!("Arbitrage processing cycle complete");

            Ok(())
        }).await;

        // result
        if let Err(e) = result {
            error!("Error running router: {:?}", e);
        }

        // Wait for specified duration before running the check again
        sleep(CHECK_INTERVAL).await;
    }
}

/// Extract pool reserves from pool data based on DEX type
fn extract_pool_reserves(pool_data: &Box<dyn std::any::Any + Send + Sync>, dex_type: dex::types::DexType) -> Option<dex::types::PoolReserves> {
    match dex_type {
        dex::types::DexType::Orca => {
            // Try to extract Orca Whirlpool data
            // This is just an example - you'll need to adjust based on actual data structure
            if let Some(orca_data) = pool_data.downcast_ref::<OrcaWhirlpoolData>() {
                Some(dex::types::PoolReserves {
                    sqrt_price: orca_data.sqrt_price,
                    tick_current_index: orca_data.tick_current_index,
                    liquidity: orca_data.liquidity,
                    fee_rate: orca_data.fee_rate,
                    tick_spacing: orca_data.tick_spacing,
                    token_a_reserves: None, // Orca doesn't use these directly
                    token_b_reserves: None,
                })
            } else {
                // If we can't downcast to Orca data, log a warning and return default
                tracing::warn!("Could not extract Orca pool reserves data");
                Some(dex::types::PoolReserves::default())
            }
        },
        dex::types::DexType::RaydiumCpmm => {
            // For CPMM-style AMMs, we need token reserves
            if let Some(cpmm_data) = pool_data.downcast_ref::<RaydiumCpmmData>() {
                Some(dex::types::PoolReserves {
                    sqrt_price: 0, // Not used by CPMM
                    tick_current_index: 0, // Not used by CPMM
                    liquidity: 0, // Not used by CPMM
                    fee_rate: cpmm_data.fee_rate,
                    tick_spacing: 0, // Not used by CPMM
                    token_a_reserves: Some(cpmm_data.token_a_amount),
                    token_b_reserves: Some(cpmm_data.token_b_amount),
                })
            } else {
                tracing::warn!("Could not extract Raydium CPMM pool reserves data");
                Some(dex::types::PoolReserves::default())
            }
        },
        // Add other DEX type handlers here
        _ => {
            // For unsupported types, just return default values for now
            tracing::warn!("Unsupported DEX type for pool reserves extraction: {:?}", dex_type);
            Some(dex::types::PoolReserves::default())
        }
    }
}

/// Get quotes from DEXes for all pools
///
/// This function takes the pool entries and returns a vector of quotes from each DEX
/// The quotes can then be used to determine arbitrage opportunities
pub fn get_dex_quotes(pool_entries: &[PoolEntry]) -> Result<Vec<dex::types::SwapQuote>, anyhow::Error> {
    let mut quotes = Vec::new();

    // Use tracing for better diagnostic information
    tracing::debug!("Getting DEX quotes for {} pools", pool_entries.len());

    for (pool_address, pool_data) in pool_entries {
        // Determine the DEX type based on the pool address
        let dex_type = dex::determine_dex_type(pool_address);
        tracing::debug!("Pool {:?} identified as DEX type: {:?}", pool_address, dex_type);

        // Extract pool reserves based on DEX type
        if let Some(pool_reserves) = extract_pool_reserves(pool_data, dex_type) {
            // Create a quoter for this DEX type
            let quoter = dex::create_dex_quoter(dex_type);

            // Get quotes for varying input amounts to better understand the price impact curve
            let input_amounts = [1_000_000u64, 10_000_000u64, 100_000_000u64]; // 1, 10, 100 units with 6 decimal places
            let slippage_bps = 30; // 0.3% slippage tolerance

            for &amount_in in &input_amounts {
                // Get quote for A->B
                match quoter.get_swap_quote(
                    pool_address,
                    &pool_reserves,
                    amount_in,
                    true, // A to B
                    slippage_bps,
                ) {
                    Ok(quote) => {
                        tracing::debug!(
                            "A->B quote for pool {:?}: {} in, {} out, {} fee, {:.4}% impact",
                            pool_address, quote.amount_in, quote.amount_out, quote.fee_amount, quote.price_impact * 100.0
                        );
                        quotes.push(quote);
                    },
                    Err(e) => {
                        tracing::warn!("Failed to get A->B quote for pool {:?}: {}", pool_address, e);
                    }
                }

                // Get quote for B->A
                match quoter.get_swap_quote(
                    pool_address,
                    &pool_reserves,
                    amount_in,
                    false, // B to A
                    slippage_bps,
                ) {
                    Ok(quote) => {
                        tracing::debug!(
                            "B->A quote for pool {:?}: {} in, {} out, {} fee, {:.4}% impact",
                            pool_address, quote.amount_in, quote.amount_out, quote.fee_amount, quote.price_impact * 100.0
                        );
                        quotes.push(quote);
                    },
                    Err(e) => {
                        tracing::warn!("Failed to get B->A quote for pool {:?}: {}", pool_address, e);
                    }
                }
            }
        } else {
            tracing::warn!("Could not extract pool reserves for pool {:?}", pool_address);
        }
    }

    tracing::info!("Generated {} quotes from {} pools", quotes.len(), pool_entries.len());
    Ok(quotes)
}

pub fn solve(pool_entries: &[PoolEntry]) -> Result<ArbitrageResult, Box<dyn std::error::Error>> {
    println!("Received {} pool entries for solving", pool_entries.len());

    let result = Python::with_gil(|py| -> PyResult<ArbitrageResult> {
        let qtrade = PyModule::import(py, "qtrade.arbitrage.core")?;

        // Problem data
        let global_indices = vec![0, 1, 2, 3];
        let local_indices = vec![
            vec![0, 1, 2, 3],
            vec![0, 1],
            vec![1, 2],
            vec![2, 3],
            vec![2, 3],
        ];
        let reserves = vec![
            vec![4.0, 4.0, 4.0, 4.0],
            vec![10.0, 1.0],
            vec![1.0, 5.0],
            vec![40.0, 50.0],
            vec![10.0, 10.0],
        ];
        let fees = vec![0.998, 0.997, 0.997, 0.997, 0.999];
        let market_value = vec![1.5, 10.0, 2.0, 3.0];

        // Convert Rust data to Python objects
        let py_global_indices = PyList::new(py, &global_indices)?;
        let py_local_indices = PyList::new(py, &local_indices)?;

        let py_reserves: Vec<_> = reserves
            .iter()
            .map(|r| PyList::new(py, r))
            .collect::<Result<Vec<_>, _>>()?;
        let py_reserves_list = PyList::new(py, &py_reserves)?;

        let py_fees = PyList::new(py, &fees)?;
        let py_market_value = PyList::new(py, &market_value)?;

        // Call the Python function with positional arguments
        let args = (
            py_global_indices,
            py_local_indices,
            py_reserves_list,
            py_fees,
            py_market_value,
        );
        let inner_result = qtrade.call_method("solve_arbitrage", args, None)?;

        // Extract results from the Python function
        let (prob, deltas, lambdas, a): (Bound<'_, PyAny>, Bound<'_, PyList>, Bound<'_, PyList>, Bound<'_, PyList>)
            = inner_result.extract()?;

        // Print results for debugging
        println!("Optimization problem status: {:?}", prob.getattr("status")?);

        // Convert PyList objects to Rust vectors
        let deltas_vec: Vec<Vec<f64>> = deltas
            .iter()
            .map(|delta_list| {
                let delta_py_list = delta_list.downcast::<PyList>().unwrap();
                delta_py_list.iter()
                    .map(|val| val.extract::<f64>().unwrap())
                    .collect::<Vec<f64>>()
            })
            .collect();

        let lambdas_vec: Vec<Vec<f64>> = lambdas
            .iter()
            .map(|lambda_list| {
                let lambda_py_list = lambda_list.downcast::<PyList>().unwrap();
                lambda_py_list.iter()
                    .map(|val| val.extract::<f64>().unwrap())
                    .collect::<Vec<f64>>()
            })
            .collect();

        let a_vec: Vec<Vec<Vec<f64>>> = a
            .iter()
            .map(|a_matrix| {
                let a_matrix_py_list = a_matrix.downcast::<PyList>().unwrap();
                a_matrix_py_list.iter()
                    .map(|row| {
                        let row_py_list = row.downcast::<PyList>().unwrap();
                        row_py_list.iter()
                            .map(|val| val.extract::<f64>().unwrap())
                            .collect::<Vec<f64>>()
                    })
                    .collect::<Vec<Vec<f64>>>()
            })
            .collect();

        println!("Converted deltas: {:?}", deltas_vec);
        println!("Converted lambdas: {:?}", lambdas_vec);
        println!("Converted matrix A: {:?}", a_vec);

        // Get the optimization problem status
        let status = prob.getattr("status")?.extract::<String>()?;

        // Create and return the arbitrage result
        let arbitrage_result = ArbitrageResult {
            deltas: deltas_vec,
            lambdas: lambdas_vec,
            a_matrices: a_vec,
            status,
        };

        Ok(arbitrage_result)
    });

    match &result {
        Ok(res) => println!("solve_arbitrage executed successfully with status: {}", res.status),
        Err(e) => println!("Error executing solve_arbitrage: {}", e),
    }

    // Convert PyResult<ArbitrageResult> to Result<ArbitrageResult, Box<dyn std::error::Error>>
    result.map_err(|e| e.into())
}

pub fn solve2() -> Result<(), Box<dyn std::error::Error>> {
    let result = Python::with_gil(|py| -> PyResult<()> {
        /*
        let sys = py.import("sys")?;
        let version: String = sys.getattr("version")?.extract()?;

        let locals = [("os", py.import("os")?)].into_py_dict(py)?;
        let code = c_str!("os.getenv('USER') or os.getenv('USERNAME') or 'Unknown'");
        let user: String = py.eval(code, None, Some(&locals))?.extract()?;

        println!("Hello {}, I'm Python {}", user, version);
        */

        let cvxpy = PyModule::import(py, "cvxpy")?;

        // Problem data
        let global_indices = vec![0, 1, 2, 3];

        // 0 = TOKEN-0
        // 1 = TOKEN-1
        // 2 = TOKEN-2
        // 3 = TOKEN-3

        let local_indices = vec![
            vec![0, 1, 2, 3], // TOKEN-0/TOKEN-1/TOKEN-2/TOKEN-3
            vec![0, 1],       // TOKEN-0/TOKEN-1
            vec![1, 2],       // TOKEN-1/TOKEN-2
            vec![2, 3],       // TOKEN-2/TOKEN-3
            vec![2, 3],       // TOKEN-2/TOKEN-3
        ];

        let reserves = vec![
            array![4.0, 4.0, 4.0, 4.0], // balancer with 4 assets in pool TOKEN-0, TOKEN-1, TOKEN-2, TOKEN-3
            array![10.0, 1.0],          // uniswapV2 TOKEN-0/TOKEN-1
            array![1.0, 5.0],           // uniswapV2 TOKEN-1/TOKEN-2
            array![40.0, 50.0],         // uniswapV2 TOKEN-2/TOKEN-3
            array![10.0, 10.0],         // constant_sum TOKEN-2/TOKEN-3
        ];

        let fees = vec![0.998, 0.997, 0.997, 0.997, 0.999];

        // "Market value" of tokens (say, in a centralized exchange)
        let market_value = vec![1.5, 10.0, 2.0, 3.0];

        // Build local-global matrices
        let n = global_indices.len();
        let m = local_indices.len();

        let mut a = Vec::new();
        for l in &local_indices {
            let n_i = l.len();
            let mut a_i = Array2::<f64>::zeros((n, n_i));
            for (i, &idx) in l.iter().enumerate() {
                a_i[[idx, i]] = 1.0;
            }
            a.push(a_i);
        }


        // Build variables

        /*
        let deltas: Vec<_> = local_indices.iter().map(|l| variable.call1((l.len(),))?.extract::<Array1<f64>>()).collect();
        let lambdas: Vec<_> = local_indices.iter().map(|l| variable.call1((l.len(),))?.extract::<Array1<f64>>()).collect();
        */

        // tender delta
        let mut deltas: Vec<Bound<'_, PyAny>> = Vec::new();
        for l in &local_indices {
            // let variable = cvxpy.getattr("Variable")?;
            let args = (l.len(),);
            let kwargs = PyDict::new(py);
            kwargs.set_item("nonneg", true)?;
            let inner_result = cvxpy.call_method(
                "Variable",
                args,
                Some(&kwargs));
            match inner_result {
                Ok(_) => {
                    println!("cvxpy delta variable created successfully.");
                    deltas.push(inner_result.unwrap());
                },
                Err(e) => {
                    println!("cvxpy delta variable creation error: {}", e);
                    return Err(e);
                }
            }
        }

        // receive lambda
        let mut lambdas: Vec<Bound<'_, PyAny>> = Vec::new();
        for l in &local_indices {
            // let variable = cvxpy.getattr("Variable")?;
            let args = (l.len(),);
            let kwargs = PyDict::new(py);
            kwargs.set_item("nonneg", true)?;
            let inner_result = cvxpy.call_method(
                "Variable",
                args,
                Some(&kwargs));
            match inner_result {
                Ok(_) => {
                    println!("cvxpy lambda variable created successfully.");
                    lambdas.push(inner_result.unwrap());
                },
                Err(e) => {
                    println!("cvxpy lambda variable creation error: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(())
    });

    Ok(())
    /*


    // Objective is to maximize "total market value" of coins out
    let obj = market_value.iter().zip(&psi).map(|(m, p)| m * p).sum::<f64>();

    // Reserves after trade
    let new_reserves: Vec<Array1<f64>> = reserves.iter()
        .zip(&fees)
        .zip(&deltas)
        .zip(&lambdas)
        .map(|(((r, &gamma_i), d), l)| r + &(gamma_i * d) - l)
        .collect();

    // Trading function constraints
    let cons = vec![
        new_reserves[0].iter().product::<f64>() >= reserves[0].iter().product::<f64>(),
        new_reserves[1].iter().product::<f64>() >= reserves[1].iter().product::<f64>(),
        new_reserves[2].iter().product::<f64>() >= reserves[2].iter().product::<f64>(),
        new_reserves[3].iter().product::<f64>() >= reserves[3].iter().product::<f64>(),
        new_reserves[4].sum() >= reserves[4].sum(),
        new_reserves[4].iter().all(|&x| x >= 0.0),
        psi.iter().all(|&x| x >= 0.0),
    ];

    // Trade Execution Ordering
    let mut current_tokens = vec![0.0; 4];
    let mut new_current_tokens = vec![0.0; 4];
    let mut tokens_required_arr = Vec::new();
    let mut tokens_required_value_arr = Vec::new();

    let pool_names = vec![
        "BALANCER 0/1/2/3",
        "UNIV2 0/1",
        "UNIV2 1/2",
        "UNIV2 2/3",
        "CONSTANT SUM 2/3",
    ];

    let permutations = (0..local_indices.len()).permutations(local_indices.len());
    let mut permutations2 = Vec::new();
    for permutation in permutations {
        permutations2.push(permutation.clone());
        current_tokens = vec![0.0; 4];
        new_current_tokens = vec![0.0; 4];
        let mut tokens_required = vec![0.0; 4];
        for &pool_id in &permutation {
            let pool = &local_indices[pool_id];
            for &global_token_id in pool {
                let local_token_index = pool.iter().position(|&x| x == global_token_id).unwrap();
                new_current_tokens[global_token_id] = current_tokens[global_token_id]
                    + (lambdas[pool_id][local_token_index] - deltas[pool_id][local_token_index]);

                if new_current_tokens[global_token_id] < 0.0
                    && new_current_tokens[global_token_id] < current_tokens[global_token_id]
                {
                    if current_tokens[global_token_id] < 0.0 {
                        tokens_required[global_token_id] +=
                            current_tokens[global_token_id] - new_current_tokens[global_token_id];
                        new_current_tokens[global_token_id] = 0.0;
                    } else {
                        tokens_required[global_token_id] += -new_current_tokens[global_token_id];
                        new_current_tokens[global_token_id] = 0.0;
                    }
                }
                current_tokens[global_token_id] = new_current_tokens[global_token_id];
            }
        }

        let tokens_required_value: Vec<f64> = tokens_required
            .iter()
            .zip(&market_value)
            .map(|(&i1, &i2)| i1 * i2)
            .collect();

        tokens_required_arr.push(tokens_required);
        tokens_required_value_arr.push(tokens_required_value.iter().sum());
    }

    let min_value = tokens_required_value_arr
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);
    let min_value_index = tokens_required_value_arr
        .iter()
        .position(|&x| (x - min_value).abs() < f64::EPSILON)
        .unwrap();

    println!("\n-------------------- ARBITRAGE TRADES + EXECUTION ORDER --------------------\n");
    for &pool_id in &permutations2[min_value_index] {
        let pool = &local_indices[pool_id];
        println!("\nTRADE POOL = {}", pool_names[pool_id]);

        for &global_token_id in pool {
            let local_token_index = pool.iter().position(|&x| x == global_token_id).unwrap();
            if (lambdas[pool_id][local_token_index] - deltas[pool_id][local_token_index]) < 0.0 {
                println!(
                    "\tTENDERING {} TOKEN {}",
                    -(lambdas[pool_id][local_token_index] - deltas[pool_id][local_token_index]),
                    global_token_id
                );
            }
        }

        for &global_token_id in pool {
            let local_token_index = pool.iter().position(|&x| x == global_token_id).unwrap();
            if (lambdas[pool_id][local_token_index] - deltas[pool_id][local_token_index]) >= 0.0 {
                println!(
                    "\tRECEIVING {} TOKEN {}",
                    lambdas[pool_id][local_token_index] - deltas[pool_id][local_token_index],
                    global_token_id
                );
            }
        }
    }

    println!("\n-------------------- REQUIRED TOKENS TO KICK-START ARBITRAGE --------------------\n");
    println!("TOKEN-0 = {}", tokens_required_arr[min_value_index][0]);
    println!("TOKEN-1 = {}", tokens_required_arr[min_value_index][1]);
    println!("TOKEN-2 = {}", tokens_required_arr[min_value_index][2]);
    println!("TOKEN-3 = {}", tokens_required_arr[min_value_index][3]);

    println!("\nUSD VALUE REQUIRED = ${}", min_value);

    println!("\n-------------------- TOKENS & VALUE RECEIVED FROM ARBITRAGE --------------------\n");
    let mut net_network_trade_tokens = vec![0.0; 4];
    let mut net_network_trade_value = vec![0.0; 4];

    for &pool_id in &permutations2[min_value_index] {
        let pool = &local_indices[pool_id];
        for &global_token_id in pool {
            let local_token_index = pool.iter().position(|&x| x == global_token_id).unwrap();
            net_network_trade_tokens[global_token_id] += lambdas[pool_id][local_token_index];
            net_network_trade_tokens[global_token_id] -= deltas[pool_id][local_token_index];
        }
    }

    for i in 0..net_network_trade_tokens.len() {
        net_network_trade_value[i] = net_network_trade_tokens[i] * market_value[i];
    }

    println!("RECEIVED {} TOKEN-0 = ${}", net_network_trade_tokens[0], net_network_trade_value[0]);
    println!("RECEIVED {} TOKEN-1 = ${}", net_network_trade_tokens[1], net_network_trade_value[1]);
    println!("RECEIVED {} TOKEN-2 = ${}", net_network_trade_tokens[2], net_network_trade_value[2]);
    println!("RECEIVED {} TOKEN-3 = ${}", net_network_trade_tokens[3], net_network_trade_value[3]);

    println!("\nSUM OF RECEIVED TOKENS USD VALUE = ${}", net_network_trade_value.iter().sum::<f64>());
    println!("CONVEX OPTIMISATION SOLVER RESULT: ${}", obj);

    Ok(())
    */
}
