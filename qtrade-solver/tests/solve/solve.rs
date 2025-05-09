use qtrade_solver::{solve, PoolEntry};
use spl_pod::solana_pubkey::Pubkey;
use std::str::FromStr;
use std::panic::AssertUnwindSafe;

#[test]
fn test_solve() {
    // Create dummy pool entries for testing
    let dummy_entries: Vec<PoolEntry> = vec![
        (
            Pubkey::from_str("11111111111111111111111111111111").unwrap(),
            Box::new(()) as Box<dyn std::any::Any + Send + Sync>
        ),
        (
            Pubkey::from_str("22222222222222222222222222222222").unwrap(),
            Box::new(()) as Box<dyn std::any::Any + Send + Sync>
        ),
        (
            Pubkey::from_str("33333333333333333333333333333333").unwrap(),
            Box::new(()) as Box<dyn std::any::Any + Send + Sync>
        ),
    ];

    // Capture the output of the solve function with the dummy pool entries
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| solve(&dummy_entries)));
    assert!(result.is_ok(), "solve function should not panic");

    // Check if the output contains expected strings
    /*
    assert!(output.contains("ARBITRAGE TRADES + EXECUTION ORDER"));
    assert!(output.contains("REQUIRED TOKENS TO KICK-START ARBITRAGE"));
    assert!(output.contains("TOKENS & VALUE RECEIVED FROM ARBITRAGE"));
    assert!(output.contains("CONVEX OPTIMISATION SOLVER RESULT"));
    */
}
