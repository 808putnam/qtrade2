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

fn start_validator() {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    println!("Current directory: {:?}", current_dir);

    println!("Starting solana-test-validator...");
    Command::new("solana-test-validator")
        .args(&[
            "--log",
        ])
        .stdout(std::fs::File::create("validator.log").expect("Failed to create log file"))
        .stderr(std::process::Stdio::from(std::fs::File::create("validator.log").expect("Failed to create log file")))
        .spawn()
        .expect("Failed to start solana-test-validator");

    println!("Waiting for the validator to start up...");
    std::thread::sleep(std::time::Duration::from_secs(5));

    println!("Airdropping 1 SOL to the default keypair...");
    Command::new("solana")
        .args(&["airdrop", "--url", "localhost", "1"])
        .output()
        .expect("Failed to airdrop SOL");

    println!("Deploying the program...");
    let output = Command::new("solana")
        .args(&[
            "program",
            "deploy",
            "--url",
            "localhost",
            "target/deploy/qtrade_arbitrage.so",
            "--program-id",
            "target/deploy/qtrade_arbitrage-keypair.json",
        ])
        .current_dir(current_dir.parent().expect("Failed to get parent directory")) // Set the working directory to the parent directory
        .output()
        .expect("Failed to deploy program");

    println!("solana program deploy stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("solana program deploy stderr: {}", String::from_utf8_lossy(&output.stderr));

    println!("Waiting for the program to finish loading...");
    std::thread::sleep(std::time::Duration::from_secs(5));

    println!("Starting solana logs...");
    Command::new("solana")
        .args(&["logs", "--url", "localhost"])
        .stdout(std::fs::File::create("solana_logs.log").expect("Failed to create log file"))
        .stderr(std::process::Stdio::from(std::fs::File::create("solana_logs.log").expect("Failed to create log file")))
        .spawn()
        .expect("Failed to start solana logs");

    println!("Allow solana logs to start...");
    std::thread::sleep(std::time::Duration::from_secs(3));

    println!("Validator setup complete.");

    let client_config = "client_config.ini";
    let pool_config = qtrade::load_cfg(&client_config.to_string()).unwrap();
    // cluster params.
    let payer = read_keypair_file(&pool_config.payer_path).expect("Failed to load pool_config.payer_path");
    // solana rpc client
    let rpc_client = RpcClient::new(pool_config.http_url.to_string());

    // anchor client.
    let anchor_config = pool_config.clone();
    let url = Cluster::Custom(anchor_config.http_url, anchor_config.ws_url);
    let wallet = read_keypair_file(&pool_config.payer_path).expect("Failed to load pool_config.payer_path");
    let anchor_client = Client::new(url, Rc::new(wallet));
    let program = anchor_client.program(pool_config.raydium_cp_program).expect("Failed to load pool_config.raydium_cp_program");
    
    create_mint("mint0_keypair.json", "mint0_authority_keypair.json", &pool_config);
    create_mint("mint1_keypair.json", "mint1_authority_keypair.json", &pool_config);

}

fn create_mint(mint_key_file: &str, mint_authority_file: &str, pool_config: &qtrade::ClientConfig) {
    let mint_keypair = qtrade::read_keypair_file(mint_key_file)
        .expect(&format!("Failed to read {} keypair", mint_key_file));
    let mint_key = mint_keypair.pubkey();
    let mint_authority_keypair = qtrade::read_keypair_file(mint_authority_file)
        .expect(&format!("Failed to read {} keypair", mint_authority_file));
    let mint_authority = mint_authority_keypair.pubkey();
    let instructions = create_and_init_mint_instr(
        pool_config,
        spl_token::id(),
        &mint_key,
        &mint_authority,
        None,
        vec![],
        9,
    ).expect("Failed to create and initialize mint");

    let payer = read_keypair_file(&pool_config.payer_path).expect("Failed to load pool_config.payer_path");
    let rpc_client = RpcClient::new(pool_config.http_url.to_string());
    let signers = vec![&payer];
    let recent_hash = rpc_client.get_latest_blockhash().expect("Failed to get latest blockhash");
    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &signers,
        recent_hash,
    );
    let signature = send_txn(&rpc_client, &txn, true).expect("Failed to send transaction");
    println!("{} created: {}", mint_key_file, signature);
}

struct ValidatorGuard;

impl Drop for ValidatorGuard {
    fn drop(&mut self) {
        println!("Killing the solana-test-validator...");
        Command::new("pkill")
            .args(&["-f", "solana-test-validator"])
            .output()
            .expect("Failed to kill solana-test-validator");
    }
}

#[test]
fn test_initialize() {
    println!("Starting");

    let _guard = ValidatorGuard;
    start_validator();

    let program_id = "HCW6PxAKrhZtWnmxRcUs66hDBeNp33jAqKxNFGadUwaQ";
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    let payer = read_keypair_file(&anchor_wallet).unwrap();

    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());

    let program_id = Pubkey::from_str(program_id).unwrap();
    let program = client.program(program_id).unwrap();

    /*
    let tx = program
        .request()
        .accounts(qtrade_arbitrage::accounts::Initialize {})
        .args(qtrade_arbitrage::instruction::Initialize {})
        .send()
        .expect("");

    println!("Your transaction signature {}", tx);
    */
}
