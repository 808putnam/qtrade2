//! Module for managing API keys and other secrets
//!
//! This module provides constant definitions for API keys and other secrets
//! that are used throughout the application. In a production environment,
//! these values would be loaded from environment variables or secure storage.

// Environment variables are loaded at runtime

// Temporal API key
pub const TEMPORAL_API_KEY: &str = "YOUR_TEMPORAL_API_KEY";

// Quicknode API key
pub const QUICKNODE_API_KEY: &str = "YOUR_QUICKNODE_API_KEY";

// Helius API key
pub const HELIUS_API_KEY: &str = "YOUR_HELIUS_API_KEY";

// Nextblock API key
pub const NEXTBLOCK_API_KEY: &str = "YOUR_NEXTBLOCK_API_KEY";

// Bloxroute API key
pub const BLOXROUTE_API_KEY: &str = "YOUR_BLOXROUTE_API_KEY";

// Nonce account pool environment variables
//
// QTRADE_NONCE_ACCOUNTS - Comma-separated list of nonce account public keys
// Example: "nonce_pubkey1,nonce_pubkey2,nonce_pubkey3"
//
// QTRADE_NONCE_AUTHORITY_SECRET - Base58 encoded private key for the nonce authority
// Example: "4xN3rAk8vJxX3KLCkGHEj9XuQej1V97LehTPrU7fK7B57UNQ8dFjeBs4fEXeB8Y26HqJVUNhdjS1mThJ9hNFkRDs"
