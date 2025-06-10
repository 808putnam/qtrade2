**code snapshot date: June 9, 2025**

# Welcome to QTrade
*A modular DeFi infrastructure platform powering crypto arbitrage, DEX aggregation, trading analytics, and more - built with indexer, router, relayer, monitor, and deploy components.*

For more information, check out the [documentation](https://808putnam.gitbook.io/qtrade).

QTrade provides the architectural building blocks for diverse DeFi applications including:
- **Crypto Arbitrage** - Cross-DEX profit maximization
- **DEX Aggregator** - Optimal routing across liquidity sources
- **Trading Trends & Narratives** - Real-time market analysis
- **Tokenized Securities** - Institutional-grade trading infrastructure
- **Copy Trading** - Social trading platform foundations

## indexer
At the core of *QTrade* is a custom bare-metal Solana RPC node, enhanced with a Yellowstone Geyser plugin to enable high-throughput data streaming. Real-time liquidity data from Raydium and Orca is captured and ingested using Yellowstone Vixen, which models DEX pool reserves with sub-second latency—providing the foundational data layer for any DeFi application.

## router
*QTrade*'s modular routing layer bridges Rust performance with Python flexibility. For arbitrage, it uses a hybrid CVXPY-based solver that computes optimal routes across multiple pools and tokens. For DEX aggregation, it implements graph-based pathfinding algorithms. The router architecture adapts to different use cases while maintaining optimal execution strategies.

## relayer
*QTrade* ensures precise and reliable executions through advanced transaction orchestration across multiple RPC providers. The relayer features Address Lookup Tables (ALTs) to execute complex multi-DEX transactions that exceed Solana's 64-account limit, a sophisticated three-tiered wallet management system (HODL/Bank/Explorer keys) for enhanced security, and comprehensive transaction simulation capabilities. Users can configure active RPC providers, enable single wallet mode for testing, and leverage atomicity guarantees through Solana Nonce accounts—ensuring only one provider succeeds in transaction placement. [Learn more about simulation](./docs/SIMULATION.md).

## monitor
*QTrade* employs comprehensive monitoring through Datadog and OpenTelemetry to ensure system reliability and performance. Key metrics track the health of the RPC node, the Yellowstone Geyser plugin, and the status of DEX pool reserves and transactions. Additionally, base-level infrastructure monitoring provides insights into CPU usage, container performance, and system health, ensuring continuous uptime and optimal operation.

## deploy
*QTrade*'s deployment is automated using a combination of GitHub Actions and Ansible. GitHub Actions orchestrate the entire deployment pipeline, ensuring seamless integration and continuous delivery. Ansible handles the configuration and provisioning of services, enabling consistent and scalable deployments across environments with minimal manual intervention.
