use std::env;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn test_full_lifecycle() {
    println!("Starting test_full_lifecycle");

    let wallet_config_path = env::var("WALLET_CONFIG_PATH").expect("WALLET_CONFIG_PATH not set");
    let vixon_config_path = env::var("VIXON_CONFIG_PATH").expect("VIXON_CONFIG_PATH not set");
    let blockchain = match env::var("BLOCKCHAIN").expect("BLOCKCHAIN not set").as_str() {
        "solana" => qtrade_runtime::Blockchain::Solana,
        "sui" => qtrade_runtime::Blockchain::Sui,
        _ => panic!("Invalid BLOCKCHAIN value"),
    };
    let solver = match env::var("SOLVER").expect("SOLVER not set").as_str() {
        "cvxpy" => qtrade_runtime::Solver::Cvxpy,
        "openqaoa" => qtrade_runtime::Solver::OpenQAOA,
        "cfmmrouter" => qtrade_runtime::Solver::CFMMRouter,
        _ => panic!("Invalid SOLVER value"),
    };

    let token = CancellationToken::new();

    match qtrade_runtime::run_qtrade(
        &wallet_config_path, 
        &vixon_config_path, 
        blockchain, 
        solver,
        token
    ).await {
        Ok(_) => println!("qtrade::run_qtrade completed successfully"),
        Err(e) => eprintln!("Error running qtrade::run_qtrade: {:?}", e),
    }
}




