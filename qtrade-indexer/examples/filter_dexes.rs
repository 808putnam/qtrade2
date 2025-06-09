use qtrade_indexer::settings::IndexerSettings;

fn main() {
    // Create default settings (all DEXes active)
    let default_settings = IndexerSettings::new();
    println!("Default settings - Active DEXes: {:?}", default_settings.active_dexes);
    println!("Default vixen config path: {}", default_settings.vixen_config_path);

    // Check if specific DEXes are active
    println!("Is Orca active? {}", default_settings.is_dex_active("orca"));
    println!("Is Raydium active? {}", default_settings.is_dex_active("raydium"));
    println!("Is Raydium CLMM active? {}", default_settings.is_dex_active("raydium-clmm"));
    println!("Is Raydium CPMM active? {}", default_settings.is_dex_active("raydium-cpmm"));
    println!("Is Unknown DEX active? {}", default_settings.is_dex_active("unknown-dex"));

    // Create settings with only specific DEXes active
    let custom_settings = IndexerSettings::new_with_dexes(vec![
        "orca".to_string(),
        "raydium".to_string(),
    ]);
    println!("\nCustom settings - Active DEXes: {:?}", custom_settings.active_dexes);
    println!("Custom vixen config path: {}", custom_settings.vixen_config_path);

    // Check if specific DEXes are active with custom settings
    println!("Is Orca active? {}", custom_settings.is_dex_active("orca"));
    println!("Is Raydium active? {}", custom_settings.is_dex_active("raydium"));
    println!("Is Raydium CLMM active? {}", custom_settings.is_dex_active("raydium-clmm"));
    println!("Is Raydium CPMM active? {}", custom_settings.is_dex_active("raydium-cpmm"));

    // Case insensitive check
    println!("\nCase insensitive check:");
    println!("Is ORCA active? {}", custom_settings.is_dex_active("ORCA"));
    println!("Is orca active? {}", custom_settings.is_dex_active("orca"));

    // Create settings with custom vixen config path
    let custom_config_settings = IndexerSettings::new_with_config(
        vec![
            "orca".to_string(),
            "raydium".to_string(),
        ],
        "custom_vixen_config.toml".to_string()
    );
    println!("\nCustom config settings - Active DEXes: {:?}", custom_config_settings.active_dexes);
    println!("Custom vixen config path: {}", custom_config_settings.vixen_config_path);
}
