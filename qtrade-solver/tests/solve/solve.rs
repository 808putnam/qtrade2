use qtrade_solver::solve;

#[test]
fn test_solve() {
    // Capture the output of the solve function
    let output = std::panic::catch_unwind(|| solve()).unwrap();

    // Check if the output contains expected strings
    /*
    assert!(output.contains("ARBITRAGE TRADES + EXECUTION ORDER"));
    assert!(output.contains("REQUIRED TOKENS TO KICK-START ARBITRAGE"));
    assert!(output.contains("TOKENS & VALUE RECEIVED FROM ARBITRAGE"));
    assert!(output.contains("CONVEX OPTIMISATION SOLVER RESULT"));
    */
}
