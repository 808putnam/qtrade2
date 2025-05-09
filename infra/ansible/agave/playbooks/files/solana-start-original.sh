#!/bin/bash
HOME_DIR="/home/ubuntu"
SOL_DIR="$HOME_DIR/Solana"
DATA_DIR="$SOL_DIR/ledger"
EXITED_BAD=${1:-true}

ACC_DIR="$SOL_DIR/solana-accounts"
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
export SOLANA_METRICS_CONFIG="host=https://metrics.solana.com:8086,db=mainnet-beta,u=mainnet-beta_write,p=password"

exec agave-validator \
 --identity "/home/ubuntu/Solana/validator-keypair.json" \
 --no-untrusted-rpc \
 --no-voting \
 --rpc-port 8899 \
 --known-validator B29xkvU2VG25Gc7CUTQu7QdTPeQtoqFoycBzngBJXXRk \
 --known-validator Hz5aLvpKScNWoe9YZWxBLrQA3qzHJivBGtfciMekk8m5 \
 --known-validator XkCriyrNwS3G4rzAXtG5B1nnvb5Ka1JtCku93VqeKAr \
 --known-validator RFLCTDRBVZTEbXrCd92jnKghYeDJARb6ByK2JnPfQmH \
 --known-validator bNWWZJ3boUwT7cyGUGD9RQknUTpFkZbmbfDA2sCd8KU \
 --dynamic-port-range 8000-8800 \
 --entrypoint entrypoint.mainnet-beta.solana.com:8001 \
 --entrypoint entrypoint2.mainnet-beta.solana.com:8001 \
 --entrypoint entrypoint3.mainnet-beta.solana.com:8001 \
 --entrypoint entrypoint4.mainnet-beta.solana.com:8001 \
 --entrypoint entrypoint5.mainnet-beta.solana.com:8001 \
 --expected-genesis-hash 5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d \
 --enable-rpc-transaction-history \
 --wal-recovery-mode skip_any_corrupted_record \
 --full-rpc-api \
 --use-snapshot-archives-at-startup always \
 --log "/home/ubuntu/Solana/log/solana-validator.log" \
 --ledger "/home/ubuntu/Solana/ledger" \
 --rpc-pubsub-enable-block-subscription \
 --limit-ledger-size 100000000 \
 --accounts "/home/ubuntu/Solana/solana-accounts" \
 --rpc-bind-address 0.0.0.0 \
 --private-rpc \
 --expected-shred-version 50093 \
 --no-port-check \
 --account-index program-id spl-token-owner \
 --geyser-plugin-config /home/ubuntu/Solana/geyser_config.json
 