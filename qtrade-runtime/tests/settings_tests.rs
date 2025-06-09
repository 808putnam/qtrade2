#[cfg(test)]
mod tests {
    use qtrade_runtime::settings;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_settings_load_from_flags() {
        // Create temporary file paths for config files
        let temp_dir = TempDir::new().unwrap();
        let wallet_path = temp_dir.path().join("wallet_config.json").to_str().unwrap().to_string();
        let vixon_path = temp_dir.path().join("vixon_config.json").to_str().unwrap().to_string();

        // Create Flags with some values set
        let flags = settings::Flags {
            config_file_path: Some(temp_dir.path().join("config.toml").to_str().unwrap().to_string()),
            vixon_config_path: Some(vixon_path.clone()),
            bloxroute_api_key: Some("test_bloxroute_key".to_string()),
            helius_api_key: None, // This will use env var or default
            nextblock_api_key: Some("test_nextblock_key".to_string()),
            quicknode_api_key: None, // This will use env var or default
            temporal_api_key: Some("test_temporal_key".to_string()),
            single_wallet: false,
            single_wallet_private_key: None,
            blockchain: Some(qtrade_runtime::Blockchain::Solana),
            router: Some(qtrade_runtime::Router::Cvxpy),
            simulate: false,
            active_rpcs: Some(vec![
                "bloxroute".to_string(),
                "helius".to_string(),
                "jito".to_string(),
                "nextblock".to_string(),
                "quicknode".to_string(),
                "solana".to_string(),
                "temporal".to_string()
            ]),
            active_dexes: Some(vec![
                "orca".to_string(),
                "raydium".to_string(),
                "raydium-cpmm".to_string(),
                "raydium-clmm".to_string(),
            ]),
        };

        // Temporarily set environment variables for testing
        env::set_var("HELIUS_API_KEY", "env_helius_key");
        env::set_var("QUICKNODE_API_KEY", "env_quicknode_key");

        // Load settings from flags and environment variables
        let settings = settings::Settings::load(flags).unwrap();

        // Verify the correct values were loaded
        assert_eq!(settings.vixon_config_path, vixon_path);
        assert_eq!(settings.bloxroute_api_key, "test_bloxroute_key");
        assert_eq!(settings.helius_api_key, "env_helius_key");
        assert_eq!(settings.nextblock_api_key, "test_nextblock_key");
        assert_eq!(settings.quicknode_api_key, "env_quicknode_key");
        assert_eq!(settings.temporal_api_key, "test_temporal_key");

        // Check that the blockchain and router were set correctly
        assert!(matches!(settings.blockchain, qtrade_runtime::Blockchain::Solana));
        assert!(matches!(settings.router, qtrade_runtime::Router::Cvxpy));

        // Clean up environment variables
        env::remove_var("HELIUS_API_KEY");
        env::remove_var("QUICKNODE_API_KEY");
    }

    #[test]
    fn test_settings_defaults() {
        // Create temporary file paths for config files
        let temp_dir = TempDir::new().unwrap();
        let wallet_path = temp_dir.path().join("wallet_config.json").to_str().unwrap().to_string();
        let vixon_path = temp_dir.path().join("vixon_config.json").to_str().unwrap().to_string();

        // Create minimal Flags with only required values
        let flags = settings::Flags {
            config_file_path: None, // Test with no config file path
            vixon_config_path: Some(vixon_path.clone()),
            blockchain: Some(qtrade_runtime::Blockchain::Solana),
            router: Some(qtrade_runtime::Router::Cvxpy),
            bloxroute_api_key: None,
            helius_api_key: None,
            nextblock_api_key: None,
            quicknode_api_key: None,
            temporal_api_key: None,
            single_wallet: false,
            single_wallet_private_key: None,
            simulate: false,
            active_rpcs: Some(vec![
                "bloxroute".to_string(),
                "helius".to_string(),
                "jito".to_string(),
                "nextblock".to_string(),
                "quicknode".to_string(),
                "solana".to_string(),
                "temporal".to_string()
            ]),
            active_dexes: Some(vec![
                "orca".to_string(),
                "raydium".to_string(),
                "raydium-cpmm".to_string(),
                "raydium-clmm".to_string(),
            ]),
        };

        // Clean environment to test defaults
        env::remove_var("BLOXROUTE_API_KEY");
        env::remove_var("HELIUS_API_KEY");
        env::remove_var("NEXTBLOCK_API_KEY");
        env::remove_var("QUICKNODE_API_KEY");
        env::remove_var("TEMPORAL_API_KEY");

        // Load settings with defaults
        let settings = settings::Settings::load(flags).unwrap();

        // Verify the default values were used
        assert_eq!(settings.bloxroute_api_key, "");
        assert_eq!(settings.helius_api_key, "");
        assert_eq!(settings.nextblock_api_key, "");
        assert_eq!(settings.quicknode_api_key, "");
        assert_eq!(settings.temporal_api_key, "");
    }
}
