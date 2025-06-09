use std::env;
use tokio::runtime::Runtime;
use once_cell::sync::OnceCell;
use serial_test::serial;

use qtrade_relayer::settings::RelayerSettings;

// We'll use a static variable to ensure tests don't conflict with each other
static RUNTIME: OnceCell<Runtime> = OnceCell::new();

fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        Runtime::new().expect("Failed to create runtime")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial]
    fn test_run_relayer_with_provided_settings() {
        // Clear environment variables first to avoid interference
        env::remove_var("BLOXROUTE_API_KEY");
        env::remove_var("HELIUS_API_KEY");
        env::remove_var("NEXTBLOCK_API_KEY");
        env::remove_var("QUICKNODE_API_KEY");
        env::remove_var("TEMPORAL_API_KEY");

        // Create test settings
        let test_settings = RelayerSettings::new(
            "test_bloxroute".to_string(),
            "test_helius".to_string(),
            "test_nextblock".to_string(),
            "test_quicknode".to_string(),
            "test_temporal".to_string(),
            false, // simulate
        );

        // Run relayer with our test settings
        let runtime = get_runtime();
        let run_result = runtime.block_on(async {
            // We'll set up a cancellation token that cancels after a short time
            // to avoid blocking indefinitely
            let token = tokio_util::sync::CancellationToken::new();
            let token_clone = token.clone();

            // Spawn a task that will cancel after a short delay
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                token_clone.cancel();
            });

            // Run relayer with our test settings and cancellation token
            let result = qtrade_relayer::run_relayer(Some(test_settings.clone()), token).await;

            // Return both the result and the settings we provided
            (result, test_settings)
        });

        // The run_relayer call might have been canceled, but that's expected
        // What we're testing is that the settings were properly set
        let (_result, settings) = run_result;

        // Get the global settings that were set by run_relayer
        let global_settings = qtrade_relayer::get_relayer_settings();

        // Verify that the global settings match what we provided
        assert_eq!(global_settings.get_bloxroute_api_key(), settings.get_bloxroute_api_key());
        assert_eq!(global_settings.get_helius_api_key(), settings.get_helius_api_key());
        assert_eq!(global_settings.get_nextblock_api_key(), settings.get_nextblock_api_key());
        assert_eq!(global_settings.get_quicknode_api_key(), settings.get_quicknode_api_key());
        assert_eq!(global_settings.get_temporal_api_key(), settings.get_temporal_api_key());
    }

    #[test]
    #[serial]
    fn test_run_relayer_with_env_settings() {
        // Set environment variables for testing
        env::set_var("BLOXROUTE_API_KEY", "env_bloxroute");
        env::set_var("HELIUS_API_KEY", "env_helius");
        env::set_var("NEXTBLOCK_API_KEY", "env_nextblock");
        env::set_var("QUICKNODE_API_KEY", "env_quicknode");
        env::set_var("TEMPORAL_API_KEY", "env_temporal");

        // Run relayer without providing settings (should use env vars)
        let runtime = get_runtime();
        let run_result = runtime.block_on(async {
            // We'll set up a cancellation token that cancels after a short time
            // to avoid blocking indefinitely
            let token = tokio_util::sync::CancellationToken::new();
            let token_clone = token.clone();

            // Spawn a task that will cancel after a short delay
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                token_clone.cancel();
            });

            // Run relayer without settings, forcing it to use env vars
            qtrade_relayer::run_relayer(None, token).await
        });

        // The run_relayer call might have been canceled, but that's expected
        // What we're testing is that the settings were properly set from env vars
        let _result = run_result;

        // Get the global settings that were set by run_relayer
        let global_settings = qtrade_relayer::get_relayer_settings();

        // Verify that the global settings match our env vars
        assert_eq!(global_settings.get_bloxroute_api_key(), "env_bloxroute");
        assert_eq!(global_settings.get_helius_api_key(), "env_helius");
        assert_eq!(global_settings.get_nextblock_api_key(), "env_nextblock");
        assert_eq!(global_settings.get_quicknode_api_key(), "env_quicknode");
        assert_eq!(global_settings.get_temporal_api_key(), "env_temporal");

        // Clean up environment variables
        env::remove_var("BLOXROUTE_API_KEY");
        env::remove_var("HELIUS_API_KEY");
        env::remove_var("NEXTBLOCK_API_KEY");
        env::remove_var("QUICKNODE_API_KEY");
        env::remove_var("TEMPORAL_API_KEY");
    }
}
