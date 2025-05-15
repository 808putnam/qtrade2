# QTrade Wallet System

This library implements a tiered wallet system for managing Solana private keys with different responsibilities.

## Tiered Key Structure

The wallet system uses three tiers of keys, each with specific responsibilities:

### 1. HODL Keys

- **Purpose**: Cold storage keys that hold the majority of funds.
- **Usage**: Only accessed to fund Bank keys when needed.
- **Security**: Highest security level, ideally kept offline or in hardware wallets.

### 2. Bank Keys

- **Purpose**: Intermediate keys used for funding Explorer keys.
- **Usage**: Transfer funds from HODL keys to Explorer keys.
- **Security**: Medium security level, accessed more frequently than HODL keys.

### 3. Explorer Keys

- **Purpose**: Transaction signing keys for network operations.
- **Usage**: Sign and submit Solana transactions.
- **Security**: Lowest security tier, frequently rotated and single-use.
- **Lifecycle**: After being used for a transaction, Explorer keys are retired, their funds are recovered to a Bank key, and they are removed from the key pool.

## Key Management System

The system provides automatic management of the key hierarchy:

1. **Fund Distribution**: HODL keys fund Bank keys, which in turn fund Explorer keys.
2. **Key Rotation**: Explorer keys are used exactly once for a transaction, then retired.
3. **Fund Recovery**: Funds from retired Explorer keys are recovered back to Bank keys.
4. **Pool Maintenance**: The system maintains pools of available keys at each tier and creates new keys as needed.

## Usage Example

```rust
// Get an Explorer keypair for transaction signing
let (explorer_pubkey, explorer_keypair) = match qtrade_wallets::get_explorer_keypair() {
    Some(keypair) => keypair,
    None => panic!("No explorer keypairs available!"),
};

// Use the keypair to sign a transaction
// ...

// Retire the keypair after use (recovers funds and removes the key)
qtrade_wallets::return_explorer_keypair(&explorer_pubkey, true)?;
```

## Benefits

- **Enhanced Security**: By using a tiered approach, most funds are kept in cold storage.
- **Single-Use Keys**: Explorer keys are used exactly once for a transaction, minimizing exposure.
- **Automatic Fund Management**: The system automatically manages the flow of funds between tiers.
- **Automatic Key Rotation**: Explorer keys are automatically rotated, ensuring fresh keys for each transaction.

## Configuration

The system can be configured via environment variables:

- `SOLANA_RPC_URL`: URL of the Solana RPC node
- `HODL_KEYS`: Comma-separated list of Base58-encoded private keys for HODL tier
- `BANK_KEYS`: Comma-separated list of Base58-encoded private keys for Bank tier
- `EXPLORER_KEYS`: Comma-separated list of Base58-encoded private keys for Explorer tier

If no Explorer keys are provided, the system will create new ones as needed.

## Metrics System

The wallet management system includes a comprehensive metrics tracking system that monitors key operations and pool status:

### Core Metrics

- **Explorer Key Counts**: Tracks explorer keys acquired, retired, created, and recovered
- **Bank Key Operations**: Tracks bank keys funded from HODL keys
- **Fund Recovery**: Measures SOL recovered from retired explorer keys
- **Key Pool Status**: Monitors the size of each key pool and available keys
- **Key Balances**: Tracks SOL balances across different key tiers

### OpenTelemetry Integration

The metrics system integrates with OpenTelemetry to export metrics to monitoring systems like DataDog:

```rust
// Initialize the wallet system (this also initializes metrics)
qtrade_wallets::init()?;

// The balancer will automatically record metrics about key operations
qtrade_wallets::balancer().await?;

// Access metrics directly
let explorer_keys_acquired = qtrade_wallets::metrics::WALLET_METRICS.explorer_keys_acquired.load(std::sync::atomic::Ordering::SeqCst);
let sol_recovered = qtrade_wallets::metrics::get_total_sol_recovered();
```

See the `examples/metrics_demo.rs` and `examples/otel_metrics_demo.rs` files for complete examples of using the metrics system.

### Available Metrics

| Metric Name | Type | Description |
|-------------|------|-------------|
| `qtrade.wallets.explorer_keys_acquired` | Counter | Number of explorer keys acquired for transaction signing |
| `qtrade.wallets.explorer_keys_retired` | Counter | Number of explorer keys retired after use |
| `qtrade.wallets.explorer_keys_created` | Counter | Number of new explorer keys created |
| `qtrade.wallets.explorer_keys_funds_recovered` | Counter | Number of explorer keys with funds recovered |
| `qtrade.wallets.bank_keys_funded` | Counter | Number of bank keys funded from HODL keys |
| `qtrade.wallets.sol_recovered` | Counter | Total SOL recovered from explorer keys |
| `qtrade.wallets.key_pool_size` | Counter | Size of each key pool by tier and status |
| `qtrade.wallets.key_balance` | Histogram | Distribution of key balances by tier |

### Usage Examples

The `examples` folder contains demonstrations of the metrics system:

- `simple_metrics_demo.rs`: Shows basic metrics API usage without requiring wallet setup
- `metrics_demo.rs`: Demonstrates metrics in a full wallet workflow
- `otel_metrics_demo.rs`: Shows metrics integration with OpenTelemetry

To run the simple metrics demo:

```bash
cargo run --example simple_metrics_demo
```
