# Simulation Feature - Developer Guide

## Overview

This guide provides technical details on how the transaction simulation feature is implemented in the QTrade codebase, intended for developers working on or extending the codebase.

> **Security & Financial Note:** Transactions run in simulation mode are processed entirely off-chain:
> - No transactions are submitted to the blockchain
> - No tokens or SOL are debited from any wallet
> - No transaction fees are paid
> - No signatures are published to the network
> - No state changes occur on the blockchain

## Implementation Architecture

The simulation feature follows these key architectural principles:

1. **Separation of concerns**: Simulation logic is encapsulated within the RPC providers
2. **Feature flag pattern**: A simple `--simulate` flag toggles between simulation and execution
3. **Code reuse**: Most transaction building code is shared between simulation and execution paths
4. **Multi-provider support**: Multiple RPC providers can be used for cross-validation

## Key Files and Components

| Component | Location | Purpose |
|-----------|----------|---------|
| CLI Flag | `/qtrade-client/src/main.rs` | Defines the `--simulate` flag |
| Settings Struct | `/qtrade-runtime/src/settings.rs` | Propagates the simulation flag |
| Relayer Settings | `/qtrade-relayer/src/settings.rs` | Makes flag available to landing logic |
| RPC Actions Trait | `/qtrade-relayer/src/rpc.rs` | Defines the `simulate_tx` method |
| Solana RPC | `/qtrade-relayer/src/rpc/solana.rs` | Primary simulation implementation |
| Helius RPC | `/qtrade-relayer/src/rpc/helius.rs` | Secondary simulation implementation |
| Nextblock RPC | `/qtrade-relayer/src/rpc/nextblock.rs` | Tertiary simulation implementation |
| Execution Logic | `/qtrade-relayer/src/lib.rs` | Chooses between simulation and execution |

## Implementation Details

### RpcActions Trait

The `RpcActions` trait defines the simulation interface:

```rust
pub trait RpcActions {
    // Other methods...

    /// Simulate a transaction and return the result
    fn simulate_tx(&self, ixs: &mut Vec<Instruction>, signer: &Keypair) -> Result<String, Box<dyn Error>> {
        // Default implementation returns an error since not all providers support simulation
        Err("Transaction simulation not supported by this RPC provider".into())
    }

    // Other methods...
}
```

### Simulation Method Implementation

Each RPC provider implements the simulation method with provider-specific logic. The core simulation implementation uses the Solana RPC client to send a `SimulateTransaction` request:

```rust
fn simulate_tx(&self, ixs: &mut Vec<Instruction>, signer: &Keypair) -> Result<String, Box<dyn Error>> {
    // Build transaction
    let blockhash = self.rpc_client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(ixs, Some(&signer.pubkey()), &[signer], blockhash);

    // Encode for RPC
    let serialized_encoded = bs58::encode(bincode::serialize(&tx).unwrap()).into_string();

    // Send simulation request
    let simulation_result: serde_json::Value = self.rpc_client.send(
        RpcRequest::SimulateTransaction,
        serde_json::json!([serialized_encoded, {
            "sigVerify": true,
            "commitment": CommitmentConfig::confirmed().commitment,
            "encoding": "jsonParsed",
        }]),
    )?;

    // Format and return result
    let pretty_result = serde_json::to_string_pretty(&simulation_result)?;
    Ok(pretty_result)
}
```

### Routing Logic

In the execution logic, we check the simulation flag to decide whether to simulate or submit the transaction:

```rust
if settings.simulate {
    info!("SIMULATION MODE: Simulating transaction instead of submitting");

    // Use various RPC providers for simulation
    match solana_rpc.simulate_tx(&mut instructions.clone(), &explorer_keypair) {
        Ok(simulation_result) => {
            info!("Transaction simulation result:");
            info!("{}", simulation_result);
            // Process simulation results...
        },
        Err(e) => {
            warn!("Failed to simulate transaction: {}", e);
            // Handle simulation error...
        }
    }

    // Early return - do not submit the transaction
    return Ok(());
}

// Regular execution logic follows...
```

## Testing Simulation

To test the simulation feature:

1. Use the `simulate_transaction.sh` script in the `/scripts` directory
2. Manually invoke with `./qtrade-client --simulate --verbose`
3. Check logs for simulation results

### Sample Test Workflow

```bash
# Build with debug symbols
cargo build -p qtrade-client

# Run with specific pool and token counts
./target/debug/qtrade-client --simulate --verbose --pool-count 3 --token-count 5

# Compare against real transaction (be careful with real funds!)
# First simulate
./target/debug/qtrade-client --simulate --verbose --pool-count 1 --token-count 2
# Then execute if simulation looks good (omit --simulate)
./target/debug/qtrade-client --verbose --pool-count 1 --token-count 2
```

## Adding Simulation to New RPC Providers

To add simulation support to a new RPC provider:

1. Implement the `simulate_tx` method of the `RpcActions` trait
2. Use the provider's specific simulation endpoint and parameters
3. Convert results to a standardized format
4. Add the provider to the simulation section in `execute_arbitrage`

## Best Practices

1. **Always check simulation before execution** in production settings
2. **Compare results across providers** for higher confidence
3. **Validate account states** in the simulation result
4. **Check for compute limits** - simulations should use < 90% of available compute
5. **Look for price impact** in simulation logs to protect against excessive slippage

## Common Simulation Issues

| Issue | Possible Causes | Solution |
|-------|----------------|----------|
| Simulation success but execution fails | State changed between simulation and execution | Minimize time gap, check for volatile markets |
| Different results across providers | Different blockchain states, network lag | Use the most conservative result |
| Simulation timeout | Complex transaction, network issues | Simplify transaction, retry or use different provider |
| Compute limit exceeded | Too many instructions in one transaction | Split into multiple transactions |
| Account not found | Using accounts that don't exist yet | Check account existence and ownership |

## Setting Compute Budget and Priority Fees

### Understanding Compute Budget Instructions

For optimal transaction execution, it's critical to properly set compute budgets and priority fees. In Solana, this is done through special instructions added to the beginning of your transaction:

```rust
use solana_sdk::compute_budget::ComputeBudgetInstruction;

// Create compute budget instructions
let cu_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(200_000); // Max compute units
let cu_price_ix = ComputeBudgetInstruction::set_compute_unit_price(10_000);  // Price in micro-lamports

// Add these instructions at the beginning of your transaction
let mut all_instructions = vec![cu_limit_ix, cu_price_ix];
all_instructions.extend(transaction_instructions);
```

### Dynamic Fee Adjustment Based on Simulation

Here's how to dynamically adjust the priority fee based on simulation results:

```rust
// First run a simulation
let simulation_result = solana_rpc.simulate_tx(&mut instructions.clone(), &signer)?;
let simulation_json: serde_json::Value = serde_json::from_str(&simulation_result)?;

// Extract compute units used
let cu_used = simulation_json["result"]["value"]["unitsConsumed"].as_u64().unwrap_or(0);

// Calculate appropriate priority fee based on expected profit and market conditions
let expected_profit = calculate_expected_profit(&simulation_json);
let market_congestion = get_current_market_congestion();
let cu_price = calculate_optimal_cu_price(cu_used, expected_profit, market_congestion);

// Create and add the compute budget instructions
let cu_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(cu_used + 10_000); // Add buffer
let cu_price_ix = ComputeBudgetInstruction::set_compute_unit_price(cu_price);

// Add these instructions at the beginning of your transaction
let mut all_instructions = vec![cu_limit_ix, cu_price_ix];
all_instructions.extend(transaction_instructions);

// Now execute the transaction with the optimized fees
solana_rpc.send_tx(&mut all_instructions, &signer)?;
```

### Sample Fee Calculation Logic

Here's a sample implementation for calculating the optimal compute unit price:

```rust
fn calculate_optimal_cu_price(
    cu_used: u64,
    expected_profit_lamports: u64,
    market_congestion: MarketCongestion
) -> u64 {
    // Base price depends on market congestion
    let base_price = match market_congestion {
        MarketCongestion::Low => 1_000,     // 1,000 micro-lamports
        MarketCongestion::Medium => 5_000,  // 5,000 micro-lamports
        MarketCongestion::High => 20_000,   // 20,000 micro-lamports
        MarketCongestion::Extreme => 50_000, // 50,000 micro-lamports
    };

    // Calculate total priority fee
    let total_priority_fee = cu_used * base_price;

    // Calculate fee as percentage of expected profit
    let fee_percentage = (total_priority_fee as f64 / expected_profit_lamports as f64) * 100.0;

    // Adjust price based on profitability
    if fee_percentage < 1.0 {
        // Very profitable trade, can afford higher fees for better confirmation chances
        return base_price * 2;
    } else if fee_percentage > 10.0 {
        // Lower profitability, reduce fees to maintain reasonable profit margin
        return base_price / 2;
    }

    // Otherwise use the base price
    base_price
}
```

### Integrating with QTrade's Simulation Framework

To integrate this with the QTrade simulation framework:

1. First run the transaction in simulation mode
2. Parse the simulation results to extract compute units used
3. Use the above logic to calculate optimal fees
4. Create a new transaction with the compute budget instructions
5. Execute the transaction with the optimized settings

This approach ensures your transactions have:
1. Sufficient compute budget to execute successfully
2. Appropriate priority fees to ensure timely inclusion
3. Fee optimization to maximize profit margins
