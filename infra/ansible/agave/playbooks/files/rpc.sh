#!/bin/bash

# Grab the latest snapshot from the Solana network and extract it to the ledger directory.
# Make sure to match the value for --version here to the version of anza that will be running
cd /home/ubuntu/scripts
python3 snapshot-finder.py --snapshot_path /home/ubuntu/Solana/ledger/ --version 2.2.12

# Standardize the location to start Solana from
cd /home/ubuntu/Solana

# Sets the CPU frequency scaling governor to "performance" mode for all CPUs on the system.
# tee writes the "performance" string to the scaling_governor file for each CPU (cpu* matches all CPUs).
# The scaling_governor file controls the CPU frequency scaling policy.
# Setting it to "performance" forces the CPU to run at its maximum frequency, improving performance
# at the cost of higher power consumption and heat generation.
# This is typically used in performance-critical applications, such as running a blockchain validator
# node, to ensure consistent and maximum CPU performance.
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Metrics environment variable for Mainnet Beta:
export SOLANA_METRICS_CONFIG="host=https://metrics.solana.com:8086,db=mainnet-beta,u=mainnet-beta_write,p=password"

# Flag Setup and references
# 1. Start with this minimal example for running a mainnet-beta "validator" node:
#    https://docs.anza.xyz/clusters/available#mainnet-beta
# 2. Then fine-tune the node for:
#    i.  Local folder locations
#    ii. RPC node setup using guidance here:
#        https://docs.anza.xyz/operations/setup-an-rpc-node
# 3. Final modifications as noted below when comparing to a real-world script:
# --no-untrusted-rpc
# - not in agave RPC node setup
# - not in Reference listing below
# - leaving out for now
#
# --known-validators
# - using ones from https://docs.anza.xyz/clusters/available#mainnet-beta
#
# --dynamic-port-range 8000-8800
# definition:
# --dynamic-port-range <MIN_PORT-MAX_PORT>
#   Range to use for dynamically assigned ports [default: 8000-10000]
# - agave RPC node setup range is 8000-8020, real-world is 8000-8800, default is 8000-10000
# - using agave RPC node setup for now
#
# --enable-rpc-transaction-history
# definition:
# Enable historical transaction info over JSON RPC, including the 'getConfirmedBlock' API. This will cause an
# increase in disk usage and IOPS
# - not in agave RPC node setup
# - leaving out for now
#
# --use-snapshot-archives-at-startup always
# definition:
# --use-snapshot-archives-at-startup <use_snapshot_archives_at_startup>
#     At startup, when should snapshot archives be extracted versus using what is already on disk?
#     Specifying "always" will always startup by extracting snapshot archives and disregard any snapshot-related
#     state already on disk. Note that starting up from snapshot archives will incur the runtime costs associated
#     with extracting the archives and rebuilding the local state.
#     Specifying "never" will never startup from snapshot archives and will only use snapshot-related state
#     already on disk. If there is no state already on disk, startup will fail. Note, this will use the latest
#     state available, which may be newer than the latest snapshot archive.
#     Specifying "when-newest" will use snapshot-related state already on disk unless there are snapshot archives
#     newer than it. This can happen if a new snapshot archive is downloaded while the node is stopped. [default:
#     when-newest]  [possible values: always, never, when-newest]
# - going with always for now
#
# --rpc-pubsub-enable-block-subscription
# definition:
# --rpc-pubsub-enable-block-subscription
#   Enable the unstable RPC PubSub `blockSubscribe` subscription
# - leaving this out for now
#
# --expected-shred-version 50093
# defintion:
# --expected-shred-version <VERSION>
#   Require the shred version be this value
# - leaving this out for now
#
# --no-port-check
# - not in agave RPC node setup
# - not in Reference listing below
# - leaving out for now
#
# --account-index program-id spl-token-owner
# Reference: https://docs.anza.xyz/operations/setup-an-rpc-node#account-indexing
# definition:
# --account-index <INDEX>...
#   Enable an accounts index, indexed by the selected account field [possible values: program-id, spl-token-
#   owner, spl-token-mint]
# - going with program-id spl-token-owner for now
#
# --geyser-plugin-config /home/ubuntu/Solana/geyser_config.json
# definition:
# --geyser-plugin-config <FILE>...
#   Specify the configuration file for the Geyser plugin.
# - going with /home/ubuntu/Solana/geyser_config.json

exec agave-validator \
    --identity "/home/ubuntu/Solana/validator-keypair.json" \
    --known-validator 7Np41oeYqPefeNQEHSv1UDhYrehxin3NStELsSKCT4K2 \
    --known-validator GdnSyH3YtwcxFvQrVVJMm1JhTS4QVX7MFsX56uJLUfiZ \
    --known-validator DE1bawNcRJB9rVm3buyMVfr8mBEoyyu73NBovf2oXJsJ \
    --known-validator CakcnaRDHka2gXyfbEd2d3xsvkJkqsLw2akB3zsN1D2S \
    --only-known-rpc \
    --full-rpc-api \
    --no-voting \
    --ledger "/home/ubuntu/Solana/ledger" \
    --accounts "/home/ubuntu/Solana/solana-accounts" \
    --log "/home/ubuntu/Solana/log/solana-validator.log" \
    --rpc-port 8899 \
    --rpc-bind-address 0.0.0.0 \
    --private-rpc \
    --dynamic-port-range 8000-8020 \
    --entrypoint entrypoint.mainnet-beta.solana.com:8001 \
    --entrypoint entrypoint2.mainnet-beta.solana.com:8001 \
    --entrypoint entrypoint3.mainnet-beta.solana.com:8001 \
    --entrypoint entrypoint4.mainnet-beta.solana.com:8001 \
    --entrypoint entrypoint5.mainnet-beta.solana.com:8001 \
    --expected-genesis-hash 5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d \
    --wal-recovery-mode skip_any_corrupted_record \
    --limit-ledger-size 100000000 \
    --use-snapshot-archives-at-startup always \
    --account-index program-id spl-token-owner \
    --geyser-plugin-config /home/ubuntu/Solana/geyser_config.json

###########
# Reference
###########
# agave-validator 2.1.17 (src:4adcd0f2; feat:3271415109, client:Agave)
# Blockchain, Rebuilt for Scale
#
# USAGE:
#     agave-validator [OPTIONS] --ledger <DIR> [SUBCOMMAND]
#
# OPTIONS:
#         --account-index-exclude-key <KEY>...
#             When account indexes are enabled, exclude this key from the index.
#
#         --account-index-include-key <KEY>...
#             When account indexes are enabled, only include specific keys in the index. This overrides --account-index-
#             exclude-key.
#         --account-index <INDEX>...
#             Enable an accounts index, indexed by the selected account field [possible values: program-id, spl-token-
#             owner, spl-token-mint]
#         --accounts <PATHS>...
#             Comma separated persistent accounts location. May be specified multiple times. [default: <LEDGER>/accounts]
#
#         --account-shrink-path <PATH>...
#             Path to accounts shrink path which can hold a compacted account set.
#
#         --accounts-db-cache-limit-mb <MEGABYTES>
#             How large the write cache for account data can become. If this is exceeded, the cache is flushed more
#             aggressively.
#         --accounts-db-test-hash-calculation
#             Enables testing of hash calculation using stores in AccountsHashVerifier. This has a computational cost.
#
#         --accounts-hash-cache-path <PATH>
#             Use PATH as accounts hash cache location [default: <LEDGER>/accounts_hash_cache]
#
#         --accounts-index-bins <BINS>
#             Number of bins to divide the accounts index into
#
#         --accounts-index-path <PATH>...
#             Persistent accounts-index location. May be specified multiple times. [default: <LEDGER>/accounts_index]
#
#         --accounts-index-scan-results-limit-mb <MEGABYTES>
#             How large accumulated results from an accounts index scan can become. If this is exceeded, the scan aborts.
#
#         --accounts-shrink-optimize-total-space <BOOLEAN>
#             When this is set to true, the system will shrink the most sparse accounts and when the overall shrink ratio
#             is above the specified accounts-shrink-ratio, the shrink will stop and it will skip all other less sparse
#             accounts. [default: true]
#         --accounts-shrink-ratio <RATIO>
#             Specifies the shrink ratio for the accounts to be shrunk. The shrink ratio is defined as the ratio of the
#             bytes alive over the  total bytes used. If the account's shrink ratio is less than this ratio it becomes a
#             candidate for shrinking. The value must between 0. and 1.0 inclusive. [default: 0.8]
#         --authorized-voter <KEYPAIR>...
#             Include an additional authorized voter keypair. May be specified multiple times. [default: the --identity
#             keypair]
#         --enable-banking-trace <BYTES>
#             Enables the banking trace explicitly, which is enabled by default and writes trace files for simulate-
#             leader-blocks, retaining up to the default or specified total bytes in the ledger. This flag can be
#             used to override its byte limit. [default: 15032385536]
#         --bind-address <HOST>
#             IP address to bind the validator ports [default: 0.0.0.0]
#
#         --block-production-method <METHOD>
#             Switch transaction scheduling method for producing ledger entries [default: central-scheduler] [possible
#             values: thread-local-multi-iterator, central-scheduler, central-scheduler-greedy]
#         --block-verification-method <METHOD>
#             Switch transaction scheduling method for verifying ledger entries [default: unified-scheduler] [possible
#             values: blockstore-processor, unified-scheduler]
#         --check-vote-account <RPC_URL>
#             Sanity check vote account state at startup. The JSON RPC endpoint at RPC_URL must expose `--full-rpc-api`
#
#         --contact-debug-interval <CONTACT_DEBUG_INTERVAL>
#             Milliseconds between printing contact debug from gossip. [default: 120000]
#
#         --cuda
#             Use CUDA
#
#         --debug-key <ADDRESS>...
#             Log when transactions are processed which reference a given key.
#
#         --dev-halt-at-slot <SLOT>
#             Halt the validator when it reaches the given slot
#
#         --disable-banking-trace
#             Disables the banking trace
#
#         --dynamic-port-range <MIN_PORT-MAX_PORT>
#             Range to use for dynamically assigned ports [default: 8000-10000]
#
#         --enable-bigtable-ledger-upload
#             Upload new confirmed blocks into a BigTable instance
#
#         --enable-extended-tx-metadata-storage
#             Include CPI inner instructions, logs, and return data in the historical transaction info stored
#
#         --enable-rpc-bigtable-ledger-storage
#             Fetch historical transaction info from a BigTable instance as a fallback to local ledger data
#
#         --enable-rpc-transaction-history
#             Enable historical transaction info over JSON RPC, including the 'getConfirmedBlock' API. This will cause an
#             increase in disk usage and IOPS
#     -n, --entrypoint <HOST:PORT>...
#             Rendezvous with the cluster at this gossip entrypoint
#
#         --etcd-cacert-file <FILE>
#             verify the TLS certificate of the etcd endpoint using this CA bundle
#
#         --etcd-cert-file <FILE>
#             TLS certificate to use when establishing a connection to the etcd endpoint
#
#         --etcd-domain-name <DOMAIN>
#             domain name against which to verify the etcd server’s TLS certificate [default: localhost]
#
#         --etcd-endpoint <HOST:PORT>...
#             etcd gRPC endpoint to connect with
#
#         --etcd-key-file <FILE>
#             TLS key file to use when establishing a connection to the etcd endpoint
#
#         --expected-bank-hash <HASH>
#             When wait-for-supermajority <x>, require the bank at <x> to have this hash
#
#         --expected-genesis-hash <HASH>
#             Require the genesis have this hash
#
#         --expected-shred-version <VERSION>
#             Require the shred version be this value
#
#         --full-rpc-api
#             Expose RPC methods for querying chain state and transaction history
#
#         --full-snapshot-archive-path <DIR>
#             Use DIR as full snapshot archives location [default: --snapshots value]
#
#         --full-snapshot-interval-slots <NUMBER>
#             Number of slots between generating full snapshots. Must be a multiple of the incremental snapshot interval.
#             Only used when incremental snapshots are enabled. [default: 25000]
#         --geyser-plugin-always-enabled <BOOLEAN>
#             Еnable Geyser interface even if no Geyser configs are specified. [default: false]
#
#         --geyser-plugin-config <FILE>...
#             Specify the configuration file for the Geyser plugin.
#
#         --gossip-host <HOST>
#             Gossip DNS name or IP address for the validator to advertise in gossip [default: ask --entrypoint, or
#             127.0.0.1 when --entrypoint is not provided]
#         --gossip-port <PORT>
#             Gossip port number for the validator
#
#         --gossip-validator <VALIDATOR IDENTITY>...
#             A list of validators to gossip with. If specified, gossip will not push/pull from from validators outside
#             this set. [default: all validators]
#         --hard-fork <SLOT>...
#             Add a hard fork at this slot
#
#     -h, --help
#             Prints help information
#
#         --health-check-slot-distance <SLOT_DISTANCE>
#             Report this validator as healthy if its latest replayed optimistically confirmed slot is within the
#             specified number of slots from the cluster's latest optimistically confirmed slot [default: 128]
#     -i, --identity <KEYPAIR>
#             Validator identity keypair
#
#         --incremental-snapshot-archive-path <DIR>
#             Use DIR as incremental snapshot archives location [default: --snapshots value]
#
#         --init-complete-file <FILE>
#             Create this file if it doesn't already exist once validator initialization is complete
#
#         --known-validator <VALIDATOR IDENTITY>...
#             A snapshot hash must be published in gossip by this validator to be accepted. May be specified multiple
#             times. If unspecified any snapshot hash will be accepted
#     -l, --ledger <DIR>
#             Use DIR as ledger location [default: ledger]
#
#         --limit-ledger-size <SHRED_COUNT>
#             Keep this amount of shreds in root slots.
#
#         --log-messages-bytes-limit <BYTES>
#             Maximum number of bytes written to the program log before truncation
#
#     -o, --log <FILE>
#             Redirect logging to the specified file, '-' for standard error. Sending the SIGUSR1 signal to the validator
#             process will cause it to re-open the log file
#         --max-genesis-archive-unpacked-size <NUMBER>
#             maximum total uncompressed file size of downloaded genesis archive [default: 10485760]
#
#         --maximum-full-snapshots-to-retain <NUMBER>
#             The maximum number of full snapshot archives to hold on to when purging older snapshots. [default: 2]
#
#         --maximum-incremental-snapshots-to-retain <NUMBER>
#             The maximum number of incremental snapshot archives to hold on to when purging older snapshots. [default: 4]
#
#         --maximum-local-snapshot-age <NUMBER_OF_SLOTS>
#             Reuse a local snapshot if it's less than this many slots behind the highest snapshot available for download
#             from other validators [default: 2500]
#         --maximum-snapshot-download-abort <MAXIMUM_SNAPSHOT_DOWNLOAD_ABORT>
#             The maximum number of times to abort and retry when encountering a slow snapshot download. [default: 5]
#
#         --minimal-snapshot-download-speed <MINIMAL_SNAPSHOT_DOWNLOAD_SPEED>
#             The minimal speed of snapshot downloads measured in bytes/second. If the initial download speed falls below
#             this threshold, the system will retry the download against a different rpc node. [default: 10485760]
#         --no-genesis-fetch
#             Do not fetch genesis from the cluster
#
#         --no-incremental-snapshots
#             Disable incremental snapshots
#
#         --no-snapshot-fetch
#             Do not attempt to fetch a snapshot from the cluster, start from a local snapshot if present
#
#         --no-voting
#             Launch validator without voting
#
#         --only-known-rpc
#             Use the RPC service of known validators only
#
#         --private-rpc
#             Do not publish the RPC port for use by others
#
#         --public-rpc-address <HOST:PORT>
#             RPC address for the validator to advertise publicly in gossip. Useful for validators running behind a load
#             balancer or proxy [default: use --rpc-bind-address / --rpc-port]
#         --public-tpu-address <HOST:PORT>
#             Specify TPU address to advertise in gossip [default: ask --entrypoint or localhost when --entrypoint is not
#             provided]
#         --public-tpu-forwards-address <HOST:PORT>
#             Specify TPU Forwards address to advertise in gossip [default: ask --entrypoint or localhostwhen --entrypoint
#             is not provided]
#         --repair-validator <VALIDATOR IDENTITY>...
#             A list of validators to request repairs from. If specified, repair will not request from validators outside
#             this set [default: all validators]
#         --require-tower
#             Refuse to start if saved tower state is not found
#
#         --restricted-repair-only-mode
#             Do not publish the Gossip, TPU, TVU or Repair Service ports. Doing so causes the node to operate in a
#             limited capacity that reduces its exposure to the rest of the cluster. The --no-voting flag is implicit when
#             this flag is enabled
#         --rocksdb-fifo-shred-storage-size <SHRED_STORAGE_SIZE_BYTES>
#             The shred storage size in bytes. The suggested value is at least 50% of your ledger storage size. If this
#             argument is unspecified, we will assign a proper value based on --limit-ledger-size. If --limit-ledger-size
#             is not presented, it means there is no limitation on the ledger size and thus
#             rocksdb_fifo_shred_storage_size will also be unbounded.
#         --rocksdb-shred-compaction <ROCKSDB_COMPACTION_STYLE>
#             Controls how RocksDB compacts shreds. *WARNING*: You will lose your Blockstore data when you switch between
#             options. Possible values are: 'level': stores shreds using RocksDB's default (level) compaction. [default:
#             level]  [possible values: level]
#         --rpc-bigtable-app-profile-id <APP_PROFILE_ID>
#             Bigtable application profile id to use in requests [default: default]
#
#         --rpc-bigtable-instance-name <INSTANCE_NAME>
#             Name of the Bigtable instance to upload to [default: solana-ledger]
#
#         --rpc-bigtable-max-message-size <BYTES>
#             Max encoding and decoding message size used in Bigtable Grpc client [default: 67108864]
#
#         --rpc-bigtable-timeout <SECONDS>
#             Number of seconds before timing out RPC requests backed by BigTable [default: 30]
#
#         --rpc-bind-address <HOST>
#             IP address to bind the RPC port [default: 127.0.0.1 if --private-rpc is present, otherwise use --bind-
#             address]
#         --rpc-blocking-threads <NUMBER>
#             Number of blocking threads to use for servicing CPU bound RPC requests (eg getMultipleAccounts) [default:
#             12]
#         --rpc-faucet-address <HOST:PORT>
#             Enable the JSON RPC 'requestAirdrop' API with this faucet address.
#
#         --rpc-max-multiple-accounts <MAX ACCOUNTS>
#             Override the default maximum accounts accepted by the getMultipleAccounts JSON RPC method [default: 100]
#
#         --rpc-max-request-body-size <BYTES>
#             The maximum request body size accepted by rpc service [default: 51200]
#
#         --rpc-niceness-adjustment <ADJUSTMENT>
#             Add this value to niceness of RPC threads. Negative value increases priority, positive value decreases
#             priority. [default: 0]
#         --rpc-port <PORT>
#             Enable JSON RPC on this port, and the next port for the RPC websocket
#
#         --rpc-pubsub-enable-block-subscription
#             Enable the unstable RPC PubSub `blockSubscribe` subscription
#
#         --rpc-pubsub-enable-vote-subscription
#             Enable the unstable RPC PubSub `voteSubscribe` subscription
#
#         --rpc-pubsub-max-active-subscriptions <NUMBER>
#             The maximum number of active subscriptions that RPC PubSub will accept across all connections. [default:
#             1000000]
#         --rpc-pubsub-notification-threads <NUM_THREADS>
#             The maximum number of threads that RPC PubSub will use for generating notifications. 0 will disable RPC
#             PubSub notifications
#         --rpc-pubsub-queue-capacity-bytes <BYTES>
#             The maximum total size of notifications that RPC PubSub will store across all connections. [default:
#             268435456]
#         --rpc-pubsub-queue-capacity-items <NUMBER>
#             The maximum number of notifications that RPC PubSub will store across all connections. [default: 10000000]
#
#         --rpc-pubsub-worker-threads <NUMBER>
#             PubSub worker threads [default: 4]
#
#         --rpc-scan-and-fix-roots
#             Verifies blockstore roots on boot and fixes any gaps
#
#         --rpc-send-transaction-also-leader
#             With `--rpc-send-transaction-tpu-peer HOST:PORT`, also send to the current leader
#
#         --rpc-send-default-max-retries <NUMBER>
#             The maximum number of transaction broadcast retries when unspecified by the request, otherwise retried until
#             expiration.
#         --rpc-send-leader-count <NUMBER>
#             The number of upcoming leaders to which to forward transactions sent via rpc service. [default: 2]
#
#         --rpc-send-retry-ms <MILLISECS>
#             The rate at which transactions sent via rpc service are retried. [default: 2000]
#
#         --rpc-send-transaction-retry-pool-max-size <NUMBER>
#             The maximum size of transactions retry pool. [default: 10000]
#
#         --rpc-send-service-max-retries <NUMBER>
#             The maximum number of transaction broadcast retries, regardless of requested value. [default:
#             18446744073709551615]
#         --rpc-send-transaction-tpu-peer <HOST:PORT>...
#             Peer(s) to broadcast transactions to instead of the current leader
#
#         --rpc-threads <NUMBER>
#             Number of threads to use for servicing RPC requests [default: 48]
#
#         --skip-preflight-health-check
#             Skip health check when running a preflight check
#
#         --skip-seed-phrase-validation
#             Skip validation of seed phrases. Use this if your phrase does not use the BIP39 official English word list
#
#         --skip-startup-ledger-verification
#             Skip ledger verification at validator bootup.
#
#         --snapshot-archive-format <ARCHIVE_TYPE>
#             Snapshot archive format to use. [default: zstd]  [possible values: zstd, lz4]
#
#         --snapshot-interval-slots <NUMBER>
#             Number of slots between generating snapshots. If incremental snapshots are enabled, this sets the
#             incremental snapshot interval. If incremental snapshots are disabled, this sets the full snapshot interval.
#             Setting this to 0 disables all snapshots. [default: 100]
#         --snapshot-packager-niceness-adjustment <ADJUSTMENT>
#             Add this value to niceness of snapshot packager thread. Negative value increases priority, positive value
#             decreases priority. [default: 0]
#         --snapshot-version <SNAPSHOT_VERSION>
#             Output snapshot version [default: 1.2.0]
#
#         --snapshot-zstd-compression-level <LEVEL>
#             The compression level to use when archiving with zstd. Higher compression levels generally produce higher
#             compression ratio at the expense of speed and memory. See the zstd manpage for more information. [default:
#             1]
#         --snapshots <DIR>
#             Use DIR as the base location for snapshots. A subdirectory named "snapshots" will be created. [default:
#             --ledger value]
#         --staked-nodes-overrides <PATH>
#             Provide path to a yaml file with custom overrides for stakes of specific identities. Overriding the amount
#             of stake this validator considers as valid for other peers in network. The stake amount is used for
#             calculating the number of QUIC streams permitted from the peer and vote packet sender stage. Format of the
#             file: `staked_map_id: {<pubkey>: <SOL stake amount>}
#         --tower <DIR>
#             Use DIR as file tower storage location [default: --ledger value]
#
#         --tower-storage <tower_storage>
#             Where to store the tower [default: file]  [possible values: file, etcd]
#
#         --tpu-coalesce-ms <MILLISECS>
#             Milliseconds to wait in the TPU receiver for packet coalescing.
#
#         --tpu-connection-pool-size <tpu_connection_pool_size>
#             Controls the TPU connection pool size per remote address [default: 1]
#
#         --tpu-disable-quic
#             Do not use QUIC to send transactions.
#
#         --tpu-enable-udp
#             Enable UDP for receiving/sending transactions.
#
#         --unified-scheduler-handler-threads <COUNT>
#             Change the number of the unified scheduler's transaction execution threads dedicated to each block,
#             otherwise calculated as cpu_cores/4 [default: 12]
#         --use-snapshot-archives-at-startup <use_snapshot_archives_at_startup>
#             At startup, when should snapshot archives be extracted versus using what is already on disk?
#             Specifying "always" will always startup by extracting snapshot archives and disregard any snapshot-related
#             state already on disk. Note that starting up from snapshot archives will incur the runtime costs associated
#             with extracting the archives and rebuilding the local state.
#             Specifying "never" will never startup from snapshot archives and will only use snapshot-related state
#             already on disk. If there is no state already on disk, startup will fail. Note, this will use the latest
#             state available, which may be newer than the latest snapshot archive.
#             Specifying "when-newest" will use snapshot-related state already on disk unless there are snapshot archives
#             newer than it. This can happen if a new snapshot archive is downloaded while the node is stopped. [default:
#             when-newest]  [possible values: always, never, when-newest]
#     -V, --version
#             Prints version information
#
#         --vote-account <ADDRESS>
#             Validator vote account public key. If unspecified, voting will be disabled. The authorized voter for the
#             account must either be the --identity keypair or set by the --authorized-voter argument
#         --wait-for-supermajority <SLOT>
#             After processing the ledger and the next slot is SLOT, wait until a supermajority of stake is visible on
#             gossip before starting PoH
#         --wal-recovery-mode <MODE>
#             Mode to recovery the ledger db write ahead log. [possible values: tolerate_corrupted_tail_records,
#             absolute_consistency, point_in_time, skip_any_corrupted_record]
#
# SUBCOMMANDS:
#     authorized-voter           Adjust the validator authorized voters
#     contact-info               Display the validator's contact info
#     exit                       Send an exit request to the validator
#     help                       Prints this message or the help of the given subcommand(s)
#     init                       Initialize the ledger directory then exit
#     monitor                    Monitor the validator
#     plugin                     Manage and view geyser plugins
#     repair-shred-from-peer     Request a repair from the specified validator
#     repair-whitelist           Manage the validator's repair protocol whitelist
#     run                        Run the validator
#     set-identity               Set the validator identity
#     set-log-filter             Adjust the validator log filter
#     set-public-address         Specify addresses to advertise in gossip
#     staked-nodes-overrides     Overrides stakes of specific node identities.
#     wait-for-restart-window    Monitor the validator for a good time to restart
#
# The default subcommand is run
