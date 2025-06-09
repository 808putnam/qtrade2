use qtrade_relayer::settings::RelayerSettings;
use std::env;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relayer_settings_creation() {
        // Test explicit creation with new()
        let settings = RelayerSettings::new(
            "test_bloxroute".to_string(),
            "test_helius".to_string(),
            "test_nextblock".to_string(),
            "test_quicknode".to_string(),
            "test_temporal".to_string(),
            false // simulate
        );

        // Verify all fields were properly set
        assert_eq!(settings.get_bloxroute_api_key(), "test_bloxroute");
        assert_eq!(settings.get_helius_api_key(), "test_helius");
        assert_eq!(settings.get_nextblock_api_key(), "test_nextblock");
        assert_eq!(settings.get_quicknode_api_key(), "test_quicknode");
        assert_eq!(settings.get_temporal_api_key(), "test_temporal");
    }

    #[test]
    fn test_relayer_settings_from_env() {
        // Set environment variables for testing
        env::set_var("BLOXROUTE_API_KEY", "env_bloxroute");
        env::set_var("HELIUS_API_KEY", "env_helius");
        env::set_var("NEXTBLOCK_API_KEY", "env_nextblock");
        env::set_var("QUICKNODE_API_KEY", "env_quicknode");
        env::set_var("TEMPORAL_API_KEY", "env_temporal");

        // Create settings from environment
        let settings = RelayerSettings::from_env();

        // Verify values were loaded from environment
        assert_eq!(settings.get_bloxroute_api_key(), "env_bloxroute");
        assert_eq!(settings.get_helius_api_key(), "env_helius");
        assert_eq!(settings.get_nextblock_api_key(), "env_nextblock");
        assert_eq!(settings.get_quicknode_api_key(), "env_quicknode");
        assert_eq!(settings.get_temporal_api_key(), "env_temporal");

        // Clean up environment variables
        env::remove_var("BLOXROUTE_API_KEY");
        env::remove_var("HELIUS_API_KEY");
        env::remove_var("NEXTBLOCK_API_KEY");
        env::remove_var("QUICKNODE_API_KEY");
        env::remove_var("TEMPORAL_API_KEY");
    }

    #[test]
    fn test_relayer_settings_defaults() {
        // Clear environment variables to test defaults
        env::remove_var("BLOXROUTE_API_KEY");
        env::remove_var("HELIUS_API_KEY");
        env::remove_var("NEXTBLOCK_API_KEY");
        env::remove_var("QUICKNODE_API_KEY");
        env::remove_var("TEMPORAL_API_KEY");

        // Create settings from environment (will use defaults)
        let settings = RelayerSettings::from_env();

        // Verify default values were used (empty strings)
        assert_eq!(settings.get_bloxroute_api_key(), "");
        assert_eq!(settings.get_helius_api_key(), "");
        assert_eq!(settings.get_nextblock_api_key(), "");
        assert_eq!(settings.get_quicknode_api_key(), "");
        assert_eq!(settings.get_temporal_api_key(), "");
    }

    #[test]
    fn test_relayer_settings_clone() {
        // Create settings
        let settings = RelayerSettings::new(
            "test_bloxroute".to_string(),
            "test_helius".to_string(),
            "test_nextblock".to_string(),
            "test_quicknode".to_string(),
            "test_temporal".to_string(),
            false // simulate
        );

        // Clone the settings
        let cloned_settings = settings.clone();

        // Verify all fields were properly cloned
        assert_eq!(cloned_settings.get_bloxroute_api_key(), "test_bloxroute");
        assert_eq!(cloned_settings.get_helius_api_key(), "test_helius");
        assert_eq!(cloned_settings.get_nextblock_api_key(), "test_nextblock");
        assert_eq!(cloned_settings.get_quicknode_api_key(), "test_quicknode");
        assert_eq!(cloned_settings.get_temporal_api_key(), "test_temporal");
    }
}
