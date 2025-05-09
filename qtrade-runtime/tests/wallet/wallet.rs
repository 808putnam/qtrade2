use std::process::Command;
use std::str::FromStr;
use std::env;
use qtrade::instructions::token_instructions::*;
use qtrade::instructions::rpc::*;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::Transaction;
use solana_client::rpc_client::RpcClient;
use std::rc::Rc;
use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig, pubkey::Pubkey, signature::read_keypair_file
    },
    Client, Cluster,
};

#[tokio::test]
#[ignore]
async fn test_jito_http_transaction() -> anyhow::Result<()> {
    use reqwest::Client;
    use serde_json::{json, Value};

    let client = HTTPClient::new(None)?;
    let lamports_to_transfer = 1_000_000;

    let from = client.public_key.unwrap();
    let to = Pubkey::from_str_const("HWEoBxYs7ssKuudEjzjmpfJVX7Dvi7wescFsVx2L5yoY");
    let keypair = client.get_keypair()?;

    // Get recent blockhash
    let block_hash = client
        .get_recent_block_hash_v2(&GetRecentBlockHashRequestV2 { offset: 0 })
        .await?
        .block_hash
        .parse::<Hash>()?;

    // Create transfer instruction
    let transfer_instruction = system_instruction::transfer(&from, &to, lamports_to_transfer);

    // Build and sign transaction
    let transaction = Transaction::new_signed_with_payer(
        &[transfer_instruction],
        Some(&from),
        &[keypair],
        block_hash,
    );

    // Serialize transaction
    let serialized_tx = bincode::serialize(&transaction)?;
    let encoded_tx = general_purpose::STANDARD.encode(serialized_tx);

    // Send to Jito API
    let http_client = Client::new();
    let response = http_client
        .post("https://mainnet.block-engine.jito.wtf/api/v1/transactions")
        .header("Content-Type", "application/json")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendTransaction",
            "params": [encoded_tx,
                {
                "encoding": "base64"
                }
            ]
        }))
        .send()
        .await?;

    let result: Value = response.json().await?;
    println!("Jito API Response: {:?}", result);

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_jito_batch_transactions() -> anyhow::Result<()> {
    let client = HTTPClient::new(None)?;
    let http_client = Client::new();
    let mut rng = rand::thread_rng();

    let from = client.public_key.unwrap();
    let to = Pubkey::from_str_const("HWEoBxYs7ssKuudEjzjmpfJVX7Dvi7wescFsVx2L5yoY");
    let keypair = client.get_keypair()?;
    
    // Single blockhash for all transactions
    let block_hash = client
        .get_recent_block_hash_v2(&GetRecentBlockHashRequestV2 { offset: 0 })
        .await?
        .block_hash
        .parse::<Hash>()?;

    // Generate 20 transactions with random small amounts
    let transactions: Vec<_> = (0..20)
        .map(|_| {
            let lamports = rng.gen_range(100..1000);
            let transfer_ix = system_instruction::transfer(&from, &to, lamports);
            
            Transaction::new_signed_with_payer(
                &[transfer_ix],
                Some(&from),
                &[keypair],
                block_hash,
            )
        })
        .collect();

    // Prepare all transaction requests
    let requests: Vec<_> = transactions
        .into_iter()
        .map(|tx| {
            let encoded_tx = general_purpose::STANDARD.encode(bincode::serialize(&tx).unwrap());
            
            http_client
                .post("https://mainnet.block-engine.jito.wtf/api/v1/transactions")
                .header("Content-Type", "application/json")
                .json(&json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "sendTransaction",
                    "params": [encoded_tx, {"encoding": "base64"}]
                }))
                .send()
        })
        .collect();

    // Send all transactions concurrently
    let responses = join_all(requests).await;
    
    // Process results
    for (i, response) in responses.into_iter().enumerate() {
        match response {
            Ok(res) => {
                let result: Value = res.json().await?;
                println!("Transaction {}: {:?}", i, result);
            }
            Err(e) => println!("Transaction {} failed: {}", i, e),
        }
    }

    Ok(())
}
