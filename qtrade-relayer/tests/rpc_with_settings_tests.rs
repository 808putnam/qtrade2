use serial_test::serial;
use qtrade_relayer::settings::RelayerSettings;
use qtrade_relayer::arbitrage::submit::create_rpc_with_settings;
use qtrade_relayer::rpc::{RpcActions, bloxroute::Bloxroute, helius::Helius, nextblock::Nextblock, quicknode::Quicknode, temporal::Temporal};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial]
    fn test_create_rpc_with_settings() {
        // Create test settings
        let settings = RelayerSettings::new(
            "test_bloxroute".to_string(),
            "test_helius".to_string(),
            "test_nextblock".to_string(),
            "test_quicknode".to_string(),
            "test_temporal".to_string(),
            false // simulate
        );

        // Create RPC providers using the settings
        let (bloxroute, helius, nextblock, quicknode, temporal) = create_rpc_with_settings(&settings);

        // Verify that each RPC provider was created with the correct API key
        assert_eq!(bloxroute.get_api_key(), "test_bloxroute");
        assert_eq!(helius.get_api_key(), "test_helius");
        assert_eq!(nextblock.get_api_key(), "test_nextblock");
        assert_eq!(quicknode.get_api_key(), "test_quicknode");
        assert_eq!(temporal.get_api_key(), "test_temporal");
    }

    #[test]
    #[serial]
    fn test_individual_rpc_creation() {
        // Create test settings
        let settings = RelayerSettings::new(
            "test_bloxroute".to_string(),
            "test_helius".to_string(),
            "test_nextblock".to_string(),
            "test_quicknode".to_string(),
            "test_temporal".to_string(),
            false // simulate
        );

        // Test each RPC provider's with_settings constructor
        let bloxroute = Bloxroute::with_settings(&settings);
        let helius = Helius::with_settings(&settings);
        let nextblock = Nextblock::with_settings(&settings);
        let quicknode = Quicknode::with_settings(&settings);
        let temporal = Temporal::with_settings(&settings);

        // Verify that each RPC provider was created with the correct API key
        assert_eq!(bloxroute.get_api_key(), "test_bloxroute");
        assert_eq!(helius.get_api_key(), "test_helius");
        assert_eq!(nextblock.get_api_key(), "test_nextblock");
        assert_eq!(quicknode.get_api_key(), "test_quicknode");
        assert_eq!(temporal.get_api_key(), "test_temporal");
    }
}
