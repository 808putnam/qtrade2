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
    let router = match env::var("ROUTER").expect("ROUTER not set").as_str() {
        "cvxpy" => qtrade_runtime::Router::Cvxpy,
        "openqaoa" => qtrade_runtime::Router::OpenQAOA,
        "cfmmrouter" => qtrade_runtime::Router::CFMMRouter,
        _ => panic!("Invalid ROUTER value"),
    };

    let token = CancellationToken::new();

    // Create flags structure with test configuration
    let flags = qtrade_runtime::settings::Flags {
        vixon_config_path: Some(vixon_config_path),
        blockchain: Some(blockchain),
        router: Some(router),
        ..Default::default()
    };

    match qtrade_runtime::run_qtrade(flags, token).await {
        Ok(_) => println!("qtrade::run_qtrade completed successfully"),
        Err(e) => eprintln!("Error running qtrade::run_qtrade: {:?}", e),
    }
}




