#!/bin/bash
# Simulation test script for qtrade
# This script demonstrates how to use the simulation feature
# NOTE: Simulations do NOT debit your wallet or make any blockchain state changes

echo "QTrade Transaction Simulation Demo"
echo "=================================="
echo
echo "IMPORTANT: Simulations run transactions in a virtual environment only."
echo "          Your wallet will NOT be debited and NO blockchain state changes occur."
echo

echo "Building qtrade-client with simulation support..."
cd "$(dirname "$0")/.."
cargo build -p qtrade-client

echo
echo "Running simulation of transaction with default parameters..."
echo "This will NOT debit your wallet or change blockchain state..."
./target/debug/qtrade-client --simulate --verbose

echo
echo "Running simulation with custom parameters..."
echo "This will NOT debit your wallet or use actual funds..."
./target/debug/qtrade-client --simulate --verbose --pool-count 3 --token-count 5

echo
echo "Advanced simulation with compute unit price settings (still NO actual funds used)..."
./target/debug/qtrade-client --simulate --verbose --pool-count 2 --cu-price 5000

echo
echo "Simulation complete. Check the logs for detailed results."
echo
echo "For more information about the simulation feature:"
echo "  - User guide: ./docs/SIMULATION.md"
echo "  - Developer guide: ./docs/SIMULATION_DEV_GUIDE.md"
echo "  - Simulation logs interpretation: See 'Understanding Simulation Logs' in SIMULATION.md"
