//! Example demonstrating the tiered key system workflow
//!
//! This example shows how the three tiers of keys interact:
//! 1. HODL keys fund Bank keys
//! 2. Bank keys fund Explorer keys
//! 3. Explorer keys are used for transactions and then retired
//! 4. Funds are recovered from retired Explorer keys

use anyhow::Result;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use std::env;
use qtrade_wallets::KeyManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Enable logging
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    println!("Demonstrating tiered key system workflow");

    // Create some test keys for each tier
    let hodl_keys = vec![
        (Keypair::new(), 1_000_000_000), // 1 SOL
        (Keypair::new(), 1_000_000_000), // 1 SOL
    ];

    let bank_keys = vec![
        (Keypair::new(), 100_000_000), // 0.1 SOL
        (Keypair::new(), 100_000_000), // 0.1 SOL
    ];

    let explorer_keys = vec![
        (Keypair::new(), 10_000_000), // 0.01 SOL
        (Keypair::new(), 10_000_000), // 0.01 SOL
    ];

    println!("HODL keys:");
    for (i, (key, _)) in hodl_keys.iter().enumerate() {
        println!("  [{i}] {}", key.pubkey());
    }

    println!("Bank keys:");
    for (i, (key, _)) in bank_keys.iter().enumerate() {
        println!("  [{i}] {}", key.pubkey());
    }

    println!("Explorer keys:");
    for (i, (key, _)) in explorer_keys.iter().enumerate() {
        println!("  [{i}] {}", key.pubkey());
    }

    // Create key manager
    // NOTE: In a real environment, you'd use a real RPC URL and fund these keys
    let key_manager = KeyManager::new(
        hodl_keys,
        bank_keys,
        explorer_keys,
        "https://api.devnet.solana.com", // Use devnet for testing
        500_000_000,  // 0.5 SOL min for HODL
        50_000_000,   // 0.05 SOL min for Bank
        5_000_000,    // 0.005 SOL min for Explorer
    );

    // --- Step 1: Simulate getting an explorer key for a transaction ---
    println!("\n--- Step 1: Get Explorer key for transaction ---");

    let (explorer_pubkey, _explorer_keypair) = match key_manager.get_explorer_keypair() {
        Some(keypair) => keypair,
        None => {
            println!("No explorer keypairs available!");
            return Ok(());
        }
    };

    println!("Got Explorer key for transaction: {}", explorer_pubkey);

    // --- Step 2: Simulate using the key for a transaction ---
    println!("\n--- Step 2: Use Explorer key for transaction ---");
    println!("Simulating transaction with key: {}", explorer_pubkey);

    // In a real scenario, we would:
    // 1. Create a transaction
    // 2. Sign it with the explorer_keypair
    // 3. Submit it to the network

    println!("Transaction completed!");

    // --- Step 3: Mark explorer key as used (retire it) ---
    println!("\n--- Step 3: Retire Explorer key after use ---");

    match key_manager.return_explorer_keypair(&explorer_pubkey, true) {
        Ok(_) => println!("Successfully retired Explorer key: {}", explorer_pubkey),
        Err(e) => println!("Failed to retire Explorer key: {}", e),
    }

    // --- Step 4: Balance key pools (clean up used keys, fund banks, create new explorers) ---
    println!("\n--- Step 4: Balance key pools ---");

    match key_manager.balance(
        2,  // min_explorer_keys
        1,  // explorer_keys_to_create
        10_000_000, // lamports_per_explorer (0.01 SOL)
        100_000_000, // lamports_per_bank (0.1 SOL)
    ).await {
        Ok(_) => println!("Successfully balanced key pools"),
        Err(e) => println!("Failed to balance key pools: {}", e),
    }

    println!("\nTiered key system workflow demonstration complete!");
    Ok(())
}
