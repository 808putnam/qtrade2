//! Sample tool to test the TOML configuration system
//!
//! This tool demonstrates how to work with the qtrade configuration system
//! to load and save TOML configuration files.

use anyhow::Result;
use qtrade_runtime::settings::{Settings, Flags};
use qtrade_runtime::{Blockchain, Router};
use std::path::Path;

fn main() -> Result<()> {
    println!("TOML Configuration System Test Tool");
    println!("===================================\n");

    // Example: Create a configuration file from default settings
    let default_settings = Settings::default();
    let example_path = Path::new("./example_config.toml");
    default_settings.create_example_config_from_current(example_path)?;
    println!("Created example configuration file at: {}", example_path.display());

    // Example: Load a configuration file
    let flags = Flags {
        config_file_path: Some(example_path.to_string_lossy().to_string()),
        blockchain: Some(Blockchain::Sui), // Override blockchain to test precedence
        ..Default::default()
    };
    let settings = Settings::load(flags)?;
    println!("\nLoaded settings from file:");
    println!("  Blockchain: {:?}", settings.blockchain);
    println!("  Router: {:?}", settings.router);

    // Example: Save current settings to a new file
    let custom_path = Path::new("./custom_config.toml");
    settings.save_to_file(custom_path)?;
    println!("\nSaved custom configuration file to: {}", custom_path.display());

    println!("\nConfiguration file testing completed successfully!");
    Ok(())
}
