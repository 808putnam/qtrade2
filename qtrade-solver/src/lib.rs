use itertools::Itertools;
use ndarray::{array, Array1, Array2};
use std::cmp::Ordering;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::ffi::c_str;
use pyo3::types::PyDict;

pub fn run() {
    println!("Hello, world!");
    // The function to launch threads, services and anything else.
    // qtrade-client does bare minimum to launch and then calls the
    // run() function, passing in any parameters it received.
}

pub fn solve() -> Result<(), Box<dyn std::error::Error>> {
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

        /*
        let psi: Array1<f64> = a.iter()
            .zip(&deltas)
            .zip(&lambdas)
            .map(|((a_i, d), l)| a_i.dot(&(l - d)));
        .sum();
        */

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

