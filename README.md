**code snapshot date: May 9, 2025**

# Welcome to qtrade
*A full stack crypto arbitrage solution - streaming infra, optimization engine, transaction builder, monitoring with Datadog and OpenTelemetry, and hybrid deployment via Ansible and GitHub Actions.*

For more information, check out the [documentation](https://808putnam.gitbook.io/qtrade).

## streaming infra
At the core of *qtrade* is a custom bare-metal Solana RPC node, enhanced with a Yellowstone Geyser plugin to enable high-throughput data streaming. Real-time liquidity data from Raydium and Orca is captured and ingested using Yellowstone Vixen, which models DEX pool reserves with sub-second latency—laying the groundwork for accurate and timely arbitrage opportunities.

## optimization engine
*qtrade*'s optimization layer bridges Rust performance with Python flexibility, using a hybrid CVXPY-based solver. Cached DEX pool reserves are streamed into the engine in near real-time, where the solver computes optimal arbitrage routes across multiple pools and tokens—maximizing profit potential while accounting for slippage, fees, and execution constraints.

## transaction builder
*qtrade* ensures precise and reliable executions by orchestrating transactions across six distinct RPC providers. Using Solana Nonce accounts, the system guarantees atomicity—ensuring that only one RPC provider will succeed in placing the transaction.

## monitoring with Datadog and OpenTelemetry
*qtrade* employs comprehensive monitoring through Datadog and OpenTelemetry to ensure system reliability and performance. Key metrics track the health of the RPC node, the Yellowstone Geyser plugin, and the status of DEX pool reserves and transactions. Additionally, base-level infrastructure monitoring provides insights into CPU usage, container performance, and system health, ensuring continuous uptime and optimal operation.

## hybrid deployment via Ansible and GitHub Actions
*qtrade*'s deployment is automated using a combination of GitHub Actions and Ansible. GitHub Actions orchestrate the entire deployment pipeline, ensuring seamless integration and continuous delivery. Ansible handles the configuration and provisioning of services, enabling consistent and scalable deployments across environments with minimal manual intervention.

