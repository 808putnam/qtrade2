#!/bin/bash
# LEGACY

# Make sure to match the value for --version here to the version of anza that will be running
cd /home/ubuntu/scripts
python3 snapshot-finder.py --snapshot_path /home/ubuntu/Solana/ledger/ --version 2.2.12

cd /home/ubuntu/Solana
nohup /home/ubuntu/solana-start.sh 2>&1 &
