//! Tests for the submit.rs module
use crate::arbitrage::submit::is_rpc_active;
use crate::settings::RelayerSettings;

#[test]
fn test_is_rpc_active() {
    // Test with all RPCs active (default)
    let settings = RelayerSettings::default();

    assert!(is_rpc_active(&settings, "solana"));
    assert!(is_rpc_active(&settings, "jito"));
    assert!(is_rpc_active(&settings, "helius"));
    assert!(is_rpc_active(&settings, "bloxroute"));
    assert!(is_rpc_active(&settings, "nextblock"));
    assert!(is_rpc_active(&settings, "quicknode"));
    assert!(is_rpc_active(&settings, "temporal"));

    // Test with specific RPCs active
    let settings = RelayerSettings::new_with_rpcs(
        "".to_string(), // bloxroute_api_key
        "".to_string(), // helius_api_key
        "".to_string(), // nextblock_api_key
        "".to_string(), // quicknode_api_key
        "".to_string(), // temporal_api_key
        vec!["solana".to_string(), "jito".to_string()], // only use Solana and Jito RPCs
        false // simulate
    );

    assert!(is_rpc_active(&settings, "solana"));
    assert!(is_rpc_active(&settings, "jito"));
    assert!(!is_rpc_active(&settings, "helius"));
    assert!(!is_rpc_active(&settings, "bloxroute"));
    assert!(!is_rpc_active(&settings, "nextblock"));
    assert!(!is_rpc_active(&settings, "quicknode"));
    assert!(!is_rpc_active(&settings, "temporal"));

    // Test with empty active_rpcs (no RPCs active)
    let settings = RelayerSettings::new_with_rpcs(
        "".to_string(), // bloxroute_api_key
        "".to_string(), // helius_api_key
        "".to_string(), // nextblock_api_key
        "".to_string(), // quicknode_api_key
        "".to_string(), // temporal_api_key
        vec![], // no RPCs active
        false // simulate
    );

    assert!(!is_rpc_active(&settings, "solana"));
    assert!(!is_rpc_active(&settings, "jito"));

    // Test case insensitivity
    let settings = RelayerSettings::new_with_rpcs(
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
        vec!["SoLaNa".to_string(), "JITO".to_string()],
        false
    );

    assert!(is_rpc_active(&settings, "solana"));
    assert!(is_rpc_active(&settings, "SOLANA"));
    assert!(is_rpc_active(&settings, "Solana"));
    assert!(is_rpc_active(&settings, "jito"));
    assert!(is_rpc_active(&settings, "JITO"));
}
