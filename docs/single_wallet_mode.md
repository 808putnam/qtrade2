# Single Wallet Mode

## Overview

Single Wallet Mode is a debugging and testing feature that simplifies the qtrade wallet system by bypassing the multi-tiered architecture and using a single wallet for all operations.

## Purpose

The standard multi-tiered wallet system (HODL, Bank, Explorer) is designed for production security and operational efficiency, but during development and testing, it can add complexity. Single Wallet Mode provides:

- Simplified debugging by using a single wallet for all operations
- Easier testing of transaction functionality without wallet management overhead
- Consistent behavior with a persistent wallet between runs (when providing the same key)

## Configuration

Single Wallet Mode can be enabled via command line flags when starting qtrade-client:

```bash
# Enable single wallet mode with a generated ephemeral key (not persistent)
./qtrade-client --blockchain solana --router cvxpy --single-wallet --vixen path/to/vixen_config.yaml --wallet path/to/wallet_config.yaml

# Enable single wallet mode with a specific private key (persistent between runs)
./qtrade-client --blockchain solana --router cvxpy --single-wallet --single-wallet-private-key <base58_private_key> --vixen path/to/vixen_config.yaml --wallet path/to/wallet_config.yaml
```

## Implementation Details

### Main Components

1. **Command-line flags**:
   - `--single-wallet` (boolean): Enables single wallet mode
   - `--single-wallet-private-key` (string, optional): Specifies a private key for the single wallet

2. **Runtime Settings**:
   - `single_wallet` (bool): Indicates whether single wallet mode is enabled
   - `single_wallet_private_key` (Option<String>): Contains the private key if provided

3. **Wallet Settings Struct**:
   - Located in `qtrade-wallets/src/lib.rs`
   - Includes configuration for single wallet mode

### Wallet Behavior Differences

When single wallet mode is enabled:

1. The HODL and Bank tiers are entirely bypassed
2. A single wallet is used for all operations
3. Key balancing and fund management are disabled
4. The same keypair is always returned by `get_explorer_keypair()`
5. Keys are never retired when returned to the pool with `return_explorer_keypair()`

### Private Key Handling

The system can operate in two modes:

1. **With provided key**: A consistent wallet is used across restarts
   - Specified with the `--single-wallet-private-key` flag
   - Provides predictable wallet identity for testing

2. **With generated key**: A new random wallet is generated on each startup
   - Used when only `--single-wallet` is specified without a private key
   - Good for isolated testing but funds will not persist between restarts

## Best Practices

1. For debugging transactions: Use single wallet mode with a provided private key
2. For quick testing: Use single wallet mode with an auto-generated key
3. For production: Disable single wallet mode to use the full tiered architecture

## Limitations

1. Security features of the tiered system are bypassed
2. Not suitable for production environments
3. If using a generated key, funds will be lost on restart
