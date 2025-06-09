# Transaction Simulation Feature

## Overview

The transaction simulation feature enables users to simulate transactions before sending them to the blockchain network. This allows testing the validity and potential outcome of transactions without actually submitting them for execution.

## Usage

To use the simulation feature, add the `--simulate` flag when running the QTrade client:

```bash
./qtrade-client --simulate [other options]
```

When this flag is enabled, transactions will be simulated through Solana RPC providers instead of being submitted for execution.

## How it Works

The simulation feature:

1. Constructs the transaction instructions based on the arbitrage opportunity
2. Creates a transaction with the appropriate keypair
3. Sends the transaction to the Solana RPC provider's simulation endpoint
4. Returns detailed results of the simulation, including potential errors or success indicators
5. Never submits the transaction to the actual blockchain network

> **Important:** Simulation is completely off-chain and does not:
> - Debit tokens or SOL from your wallet
> - Submit any transactions to validators
> - Make any changes to blockchain state
> - Require payment of transaction fees

## Benefits

- **Risk-free testing**: Validate transactions without spending gas or risking funds
- **Debugging**: Identify potential transaction errors before spending resources
- **Development**: Test new features or strategies in a safe environment
- **Gas estimation**: Determine transaction costs without executing the transaction
- **Validation**: Ensure your transaction will be accepted by the network before sending it

## Simulation Results

The simulation results include:

- Transaction execution status
- Error messages (if any)
- Account state changes
- Logs produced during simulation
- Compute units consumed
- Return data

### Understanding Simulation Logs

The logs produced during simulation are identical to those that would be generated during actual blockchain execution. These logs come directly from the Solana programs being executed in the simulation environment and can provide valuable insights:

```json
{
  "result": {
    "context": {
      "slot": 234732847
    },
    "value": {
      "err": null,
      "logs": [
        "Program 11111111111111111111111111111111 invoke [1]",
        "Program 11111111111111111111111111111111 success",
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [1]",
        "Program log: Instruction: Transfer",
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4523 of 200000 compute units",
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
        "Program CURVGoZn8zycx6FXwwevgBTB2gVvdbGTEpvMJDbgs2t4 invoke [1]",
        "Program log: Instruction: Swap",
        "Program log: Swap executed successfully. Input: 1000000 SOL, Output: 23651078 USDC",
        "Program CURVGoZn8zycx6FXwwevgBTB2gVvdbGTEpvMJDbgs2t4 consumed 56721 of 200000 compute units",
        "Program CURVGoZn8zycx6FXwwevgBTB2gVvdbGTEpvMJDbgs2t4 success"
      ],
      "accounts": null,
      "unitsConsumed": 61244,
      "returnData": null
    }
  }
}
```

#### Key Elements in Simulation Logs

1. **Program invocations and completions:**
   - `Program <ADDRESS> invoke [<DEPTH>]`: Indicates a program being called
   - `Program <ADDRESS> success` or `Program <ADDRESS> failed`: Shows the outcome

2. **Program-specific logs:**
   - `Program log: <MESSAGE>`: Custom messages emitted by programs
   - These often contain critical information about token transfers, slippage, prices, etc.

3. **Compute unit consumption:**
   - `Program <ADDRESS> consumed <AMOUNT> of <LIMIT> compute units`
   - Helps identify performance bottlenecks or compute budget issues

4. **Error information:**
   - Detailed error messages for debugging failed transactions
   - Common errors include "insufficient funds", "slippage tolerance exceeded", etc.

#### Interpreting Simulation Logs for Arbitrage

For arbitrage transactions, pay special attention to:

- Token transfer amounts between pools
- Slippage values and limits
- Price impact calculations
- Compute unit consumption (ensure it's below the limit)
- Any error messages related to pool state or token accounts

## Supported RPC Providers

The simulation feature currently supports the following RPC providers:

1. **Solana RPC** - The primary simulation provider with full support
2. **Helius RPC** - Enhanced transaction simulation with additional metrics
3. **Nextblock RPC** - Provides simulation with custom transaction prioritization

Each provider may return slightly different simulation results, which can be helpful for cross-validation.

## Limitations

- The simulated state might differ slightly from the actual blockchain state at execution time
- Some transaction failures can only be detected during actual execution
- Complex transaction interactions might not be fully captured in simulation
- Different RPC providers may have different rate limits for simulation requests

## Advanced Usage

For developers working on extending the simulation feature or integrating it with other components, refer to the [Simulation Developer Guide](./SIMULATION_DEV_GUIDE.md) for technical details on:

- Implementation architecture
- Key components and files
- Adding simulation support to new RPC providers
- Testing strategies
- Best practices for simulation-based development

## Future Enhancements

- Support for additional RPC providers (Quicknode, Jito, Temporal, etc.)
- Enhanced simulation result formatting and visualization
- Comparison view across multiple provider simulation results
- Integration with testing frameworks
- Automatic simulation before actual transaction submission
- Cost and resource estimation based on simulation results
- Simulation result storage for analysis and optimization

## Debugging with Simulation

Simulation is a powerful debugging tool for identifying issues before they occur on the blockchain. Here are some common simulation scenarios and how to interpret them:

### Common Simulation Error Patterns

#### 1. Insufficient Funds Errors

```json
{
  "err": {
    "InstructionError": [
      0,
      {
        "Custom": 1
      }
    ]
  },
  "logs": [
    "Program 11111111111111111111111111111111 invoke [1]",
    "Program log: Error: insufficient funds",
    "Program 11111111111111111111111111111111 failed: custom program error: 0x1"
  ]
}
```

This indicates that the account doesn't have enough tokens for the transaction. Check:
- Token balances in the source accounts
- Transaction fees (for SOL transactions)
- Whether you're accounting for all fees in the transaction

#### 2. Slippage Tolerance Exceeded

```json
{
  "err": {
    "InstructionError": [
      2,
      {
        "Custom": 6001
      }
    ]
  },
  "logs": [
    "Program CURVGoZn8zycx6FXwwevgBTB2gVvdbGTEpvMJDbgs2t4 invoke [1]",
    "Program log: Instruction: Swap",
    "Program log: Error: SlippageToleranceExceeded",
    "Program log: Expected output: 24000000, Actual output: 23651078",
    "Program CURVGoZn8zycx6FXwwevgBTB2gVvdbGTEpvMJDbgs2t4 failed: custom program error: 0x1771"
  ]
}
```

This shows that the transaction would fail due to price movement exceeding your specified slippage tolerance. Options:
- Increase slippage tolerance (with caution)
- Execute during periods of lower volatility
- Split the transaction into smaller amounts

#### 3. Account Validation Errors

```json
{
  "err": {
    "InstructionError": [
      1,
      "InvalidAccountData"
    ]
  },
  "logs": [
    "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [1]",
    "Program log: Error: Account not associated with this mint",
    "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA failed: invalid account data"
  ]
}
```

This indicates an issue with account validation, such as:
- Using the wrong token account for a mint
- Incorrect account ownership
- Missing account initialization

### Cross-Provider Simulation Comparison

One powerful debugging technique is to compare simulation results across different RPC providers. For example:

```
Solana RPC: Transaction would succeed, 61244 compute units consumed
Helius RPC: Transaction would succeed, 61251 compute units consumed
Nextblock RPC: Transaction would succeed, 61244 compute units consumed
```

If all providers agree, you can have higher confidence in the simulation results. If they disagree, it might indicate:
- Different blockchain state views across providers
- Provider-specific simulation limitations
- Network congestion affecting one provider

### Optimizing Based on Simulation

The simulation output can help optimize your transactions, particularly regarding Solana's fee structure and compute budget:

#### Understanding Transaction Fees on Solana

Solana transaction fees consist of two components:
1. **Base Fee**: A fixed fee per signature (currently 5000 lamports per signature)
2. **Priority Fee**: A variable fee based on compute units used and the price you're willing to pay

The **total fee** is calculated as:
```
Total Fee = Base Fee + Priority Fee
Priority Fee = Compute Units Used × Compute Unit Price
```

#### Compute Units and Priority Fees

- **Compute Units (CU)**: A measure of computational resources used by your transaction
  - Each transaction has a maximum compute unit limit (default: 200,000 CU)
  - Simulation shows exactly how many CUs your transaction will consume
  - Example: A complex swap might use ~60,000 CU while a simple transfer uses ~1,500 CU

- **Compute Unit Price (CUP)**: The price you're willing to pay per compute unit (in micro-lamports)
  - Higher CUP = higher priority for validators to include your transaction
  - Default is typically 1,000 micro-lamports per CU
  - During network congestion, you may need to increase this to 10,000+ micro-lamports

- **Setting Compute Budget**: Use these instructions in your transaction:
  ```rust
  ComputeBudgetInstruction::set_compute_unit_limit(200_000) // Set max CUs allowed
  ComputeBudgetInstruction::set_compute_unit_price(10_000)  // Set price in micro-lamports
  ```

#### Example: Using Simulation to Calculate Optimal Fees

Consider this simulation result snippet:

```json
{
  "result": {
    "value": {
      "unitsConsumed": 61244,
      "logs": [
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4523 of 200000 compute units",
        "Program CURVGoZn8zycx6FXwwevgBTB2gVvdbGTEpvMJDbgs2t4 consumed 56721 of 200000 compute units"
      ]
    }
  }
}
```

Here's how to use this information to optimize your transaction:

1. **Analyze compute unit usage**:
   - Total: 61,244 CU (30% of default 200,000 limit - good headroom)
   - Token program: 4,523 CU
   - DEX program: 56,721 CU (this is where most computation happens)

2. **Fee calculation under different network conditions**:
   | Network Congestion | Recommended CU Price | Priority Fee Calculation | Total Priority Fee |
   |--------------------|----------------------|---------------------------|-------------------|
   | Low                | 1,000 micro-lamports | 61,244 × 1,000           | 0.061244 SOL      |
   | Medium             | 5,000 micro-lamports | 61,244 × 5,000           | 0.30622 SOL       |
   | High               | 20,000 micro-lamports | 61,244 × 20,000         | 1.22488 SOL       |
   | Extreme (MEV)      | 50,000+ micro-lamports | 61,244 × 50,000        | 3.0622+ SOL       |

3. **Profit threshold decision**:
   - If your expected arbitrage profit from simulation is 5 SOL
   - Medium congestion fee (0.30622 SOL) gives you 5 - 0.30622 = 4.69378 SOL profit
   - High congestion fee (1.22488 SOL) gives you 5 - 1.22488 = 3.77512 SOL profit
   - Decision: If confirmation speed is critical, the high congestion fee still maintains good profitability

This simulation-based approach allows you to make informed decisions about transaction fees and ensure your arbitrage transactions are competitive enough to be included by validators while maximizing your profit margin.

#### Optimization Strategies Using Simulation

1. **Compute Unit Optimization**:
   - Simulation shows exact CUs consumed (e.g., `"unitsConsumed": 61244`)
   - If close to limit, simplify transaction or increase limit
   - If using much less than limit, consider bundling more operations

2. **Priority Fee Optimization**:
   - Calculate required priority fee: `CUs_used × CU_price`
   - Example: 61,244 CU × 10,000 micro-lamports = 612,440,000 micro-lamports = 0.61244 SOL
   - For arbitrage, adjust CU price based on:
     - Expected profit (higher profit = can afford higher fees)
     - Network congestion (higher congestion = need higher fees)
     - Competing transactions (MEV opportunities = need higher fees)

3. **Revenue Prediction**:
   - Use exact output amounts from simulation to calculate expected profit
   - Subtract total fees to get net profit
   - Example: If swap shows 23.65 USDC output from 1 SOL input with 0.01 SOL fees, net profit = 23.65 - (1 + 0.01) = 22.64 USDC

## Real-World Arbitrage Example

Let's walk through a complete example of using simulation to optimize an arbitrage transaction:

### Scenario
- You've identified a potential arbitrage opportunity between Raydium and Orca for SOL/USDC
- Your simulation shows:
  - Buy 10 SOL on Raydium for 1,000 USDC (100 USDC/SOL)
  - Sell 10 SOL on Orca for 1,030 USDC (103 USDC/SOL)
  - Expected profit: 30 USDC (before fees)

### Step 1: Run Transaction Simulation
```bash
./qtrade-client --simulate --verbose --pool-count 2 --token-count 2
```

### Step 2: Analyze Simulation Results

```json
{
  "result": {
    "value": {
      "err": null,
      "logs": [
        "Program ComputeBudget111111111111111111111111111111 invoke [1]",
        "Program ComputeBudget111111111111111111111111111111 success",
        "Program ComputeBudget111111111111111111111111111111 invoke [1]",
        "Program ComputeBudget111111111111111111111111111111 success",
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [1]",
        "Program log: Instruction: Transfer",
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4523 of 200000 compute units",
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
        "Program RaydiumRaYdMmqZombv3oGB9jVw9UPGVMWgYAuhQrzUQ invoke [1]",
        "Program log: Instruction: Swap",
        "Program log: Buy 10 SOL with 1000 USDC",
        "Program RaydiumRaYdMmqZombv3oGB9jVw9UPGVMWgYAuhQrzUQ consumed 42721 of 200000 compute units",
        "Program RaydiumRaYdMmqZombv3oGB9jVw9UPGVMWgYAuhQrzUQ success",
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [1]",
        "Program log: Instruction: Transfer",
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4523 of 200000 compute units",
        "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
        "Program whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc invoke [1]",
        "Program log: Instruction: Swap",
        "Program log: Sell 10 SOL for 1030 USDC",
        "Program whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc consumed 38721 of 200000 compute units",
        "Program whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc success"
      ],
      "unitsConsumed": 90488,
      "returnData": null
    }
  }
}
```

### Step 3: Calculate Fees and Profit Margin

1. **Compute Units Used**: 90,488 CUs
2. **Base Transaction Fee**: 0.000005 SOL (5,000 lamports) per signature
   - Assuming 2 signatures: 0.00001 SOL
3. **Priority Fee Calculation**:
   - Current market congestion: Medium
   - Suggested compute unit price: 5,000 micro-lamports
   - Priority fee: 90,488 × 5,000 = 452,440,000 micro-lamports = 0.45244 SOL
4. **Total Fee**: 0.00001 + 0.45244 = 0.45245 SOL
5. **SOL/USDC Price**: ~100 USDC/SOL
6. **Fee in USDC**: 0.45245 × 100 = ~45.245 USDC
7. **Net Profit Calculation**: 30 - 45.245 = -15.245 USDC (**LOSS**)

### Step 4: Optimize the Transaction

Based on simulation, this arbitrage isn't profitable at current fee levels. Options:

1. **Wait for better opportunity**: Monitor for larger price discrepancies
2. **Increase transaction size**: Scale up the transaction (if liquidity allows) to reduce fee impact
3. **Lower priority fee**: Use 2,000 micro-lamports CU price instead
   - New priority fee: 90,488 × 2,000 = 180,976,000 micro-lamports = 0.18098 SOL
   - New total fee in USDC: ~18.1 USDC
   - New net profit: 30 - 18.1 = 11.9 USDC (now profitable)
4. **Split the transaction**: If possible, execute the most profitable pairs first

### Step 5: Re-Simulate with New Parameters

```bash
./qtrade-client --simulate --verbose --pool-count 2 --token-count 2 --cu-limit 100000 --cu-price 2000
```

### Step 6: Execute or Reject

If the resimulated transaction shows a positive profit margin:
```bash
./qtrade-client --verbose --pool-count 2 --token-count 2 --cu-limit 100000 --cu-price 2000
```

Otherwise, skip this opportunity and wait for better market conditions.

This example demonstrates how simulation can prevent unprofitable arbitrage executions and optimize transaction parameters to ensure profitability.
