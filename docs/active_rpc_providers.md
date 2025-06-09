# Active RPC Provider Selection

This document explains the Active RPC Provider feature, which allows controlling which RPC providers should be used for transaction submissions.

## Overview

The qtrade architecture supports multiple RPC providers for transaction submissions. By default, all available RPC providers are used to maximize the chances of successful transaction submission. However, there may be cases where you want to restrict which RPC providers are used, such as:

- When testing with specific providers
- When certain providers are experiencing issues
- When you have API quotas to manage
- To optimize for specific provider capabilities

This feature allows you to specify which RPC providers should be active via command-line arguments, environment variables, or configuration files.

## Available RPC Providers

The following RPC providers are supported:

- `bloxroute` - Bloxroute RPC
- `helius` - Helius RPC
- `jito` - Jito RPC
- `nextblock` - Nextblock RPC
- `quicknode` - Quicknode RPC
- `solana` - Solana RPC
- `temporal` - Temporal RPC
- `triton` - Triton RPC

## Usage

### Command-line Arguments

You can specify active RPC providers using the `--active-rpcs` flag:

```bash
cargo run -p qtrade-client -- --active-rpcs solana,jito
```

This will only use the Solana and Jito RPC providers for transaction submissions.

### Environment Variables

You can also set active RPC providers using the `QTRADE_ACTIVE_RPCS` environment variable:

```bash
export QTRADE_ACTIVE_RPCS=solana,helius,jito
cargo run -p qtrade-client
```

### Configuration File

In your configuration file (`qtrade.toml`), you can specify active RPC providers:

```toml
active_rpcs = ["solana", "jito", "helius"]
```

## Default Behavior

If you don't specify active RPC providers, all providers will be active by default. This ensures backward compatibility with existing setups.

## Implementation Details

The feature works by:

1. Defining an `RpcProvider` enum in `qtrade-runtime` with all available providers
2. Adding an `active_rpcs` field to the `Settings` struct to store the active providers
3. Extending the command-line and environment variable parsing to accept active RPC lists
4. Passing the active RPC list from `Settings` to `RelayerSettings`
5. Adding an `is_rpc_active` helper function that checks if a specific RPC is enabled
6. Conditionally using RPC providers in the transaction submission process based on the active list

## Best Practices

- For maximum redundancy, use all RPC providers (default behavior)
- For targeted testing, specify only the providers you want to test
- For production environments with specific requirements, configure appropriate providers
- When debugging transaction submission issues, try isolating specific providers
