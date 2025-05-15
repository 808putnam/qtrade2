//! Blockchain Development Examples
//!
//! This module demonstrates Rust patterns specific to blockchain development:
//! - Cryptographic primitives and operations
//! - Binary data handling
//! - Account data serialization/deserialization
//! - Public/private key operations
//! - Transaction building and signing
//! - Program (smart contract) interfaces

use std::convert::TryInto;
use std::fmt;
use serde::{Deserialize, Serialize};

/// Problem: Implement a simple cryptographic hash function
///
/// Create a function that computes a SHA-256 hash
pub fn sha256_hash(data: &[u8]) -> [u8; 32] {
    use sha2::{Sha256, Digest};

    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();

    result.into()
}

/// Problem: Create a reusable data structure for public keys
///
/// Define a type for representing public keys with validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey([u8; 32]);

impl PublicKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != 32 {
            return Err(format!("Invalid public key length: {}", bytes.len()));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(bytes);
        Ok(PublicKey(key))
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_base58(&self) -> String {
        bs58::encode(&self.0).into_string()
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_base58())
    }
}

impl TryFrom<&[u8]> for PublicKey {
    type Error = String;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::from_bytes(bytes)
    }
}

/// Problem: Implement cryptographic key pair generation
///
/// Create a function to generate Ed25519 key pairs
pub fn generate_keypair() -> (Vec<u8>, PublicKey) {
    use ed25519_dalek::{Keypair, Signer, Verifier};
    use rand::rngs::OsRng;

    // Generate a new key pair
    let mut csprng = OsRng{};
    let keypair = Keypair::generate(&mut csprng);

    // Extract secret and public key
    let secret = keypair.secret.as_bytes().to_vec();
    let public = PublicKey::from_bytes(keypair.public.as_bytes())
        .expect("Invalid public key");

    (secret, public)
}

/// Problem: Sign and verify messages
///
/// Create functions for signing and verifying messages
pub fn sign_message(message: &[u8], secret_key: &[u8]) -> Result<Vec<u8>, String> {
    use ed25519_dalek::{Keypair, Signer, SecretKey, PublicKey as Ed25519PublicKey};

    // Create keypair from secret key
    let secret = SecretKey::from_bytes(secret_key)
        .map_err(|e| format!("Invalid secret key: {}", e))?;

    // Derive public key from secret
    let public = Ed25519PublicKey::from(&secret);
    let keypair = Keypair { secret, public };

    // Sign the message
    let signature = keypair.sign(message);

    Ok(signature.to_bytes().to_vec())
}

pub fn verify_signature(message: &[u8], signature: &[u8], public_key: &PublicKey) -> Result<bool, String> {
    use ed25519_dalek::{Signature, Verifier, PublicKey as Ed25519PublicKey};

    // Parse the signature
    let sig = Signature::from_bytes(signature)
        .map_err(|e| format!("Invalid signature: {}", e))?;

    // Parse the public key
    let pk = Ed25519PublicKey::from_bytes(public_key.as_bytes())
        .map_err(|e| format!("Invalid public key: {}", e))?;

    // Verify the signature
    match pk.verify(message, &sig) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Problem: Create a binary serialization format for account data
///
/// Define a structure for account data and implement serialization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenAccount {
    pub owner: [u8; 32],
    pub mint: [u8; 32],
    pub amount: u64,
    pub delegate: Option<[u8; 32]>,
    pub is_frozen: bool,
    pub is_initialized: bool,
}

impl TokenAccount {
    pub fn new(owner: [u8; 32], mint: [u8; 32]) -> Self {
        TokenAccount {
            owner,
            mint,
            amount: 0,
            delegate: None,
            is_frozen: false,
            is_initialized: true,
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, String> {
        bincode::serialize(self)
            .map_err(|e| format!("Failed to serialize: {}", e))
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, String> {
        bincode::deserialize(data)
            .map_err(|e| format!("Failed to deserialize: {}", e))
    }
}

/// Problem: Implement a transaction data structure
///
/// Create a structure representing blockchain transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub signatures: Vec<Vec<u8>>,
    pub message: TransactionMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMessage {
    pub header: MessageHeader,
    pub account_keys: Vec<[u8; 32]>,
    pub recent_blockhash: [u8; 32],
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    pub num_required_signatures: u8,
    pub num_readonly_signed_accounts: u8,
    pub num_readonly_unsigned_accounts: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    pub program_id_index: u8,
    pub accounts: Vec<u8>,
    pub data: Vec<u8>,
}

impl Transaction {
    pub fn new(
        fee_payer: &PublicKey,
        recent_blockhash: [u8; 32],
        instructions: Vec<Instruction>,
        account_keys: Vec<[u8; 32]>,
    ) -> Self {
        let header = MessageHeader {
            num_required_signatures: 1,
            num_readonly_signed_accounts: 0,
            num_readonly_unsigned_accounts: 0,
        };

        let message = TransactionMessage {
            header,
            account_keys,
            recent_blockhash,
            instructions,
        };

        Transaction {
            signatures: vec![],
            message,
        }
    }

    pub fn sign(&mut self, keypair: &[u8]) -> Result<(), String> {
        let message_data = bincode::serialize(&self.message)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;

        let signature = sign_message(&message_data, keypair)?;
        self.signatures.push(signature);

        Ok(())
    }

    pub fn serialize(&self) -> Result<Vec<u8>, String> {
        bincode::serialize(self)
            .map_err(|e| format!("Failed to serialize transaction: {}", e))
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, String> {
        bincode::deserialize(data)
            .map_err(|e| format!("Failed to deserialize transaction: {}", e))
    }
}

/// Problem: Define a program interface for token operations
///
/// Create structures for token program instructions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenInstruction {
    InitializeAccount,
    Transfer { amount: u64 },
    MintTo { amount: u64 },
    Burn { amount: u64 },
    Approve { amount: u64 },
    Revoke,
    SetAuthority { authority_type: AuthorityType, new_authority: Option<[u8; 32]> },
    CloseAccount,
    FreezeAccount,
    ThawAccount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthorityType {
    MintTokens,
    FreezeAccount,
    AccountOwner,
    CloseAccount,
}

impl TokenInstruction {
    pub fn pack(&self) -> Vec<u8> {
        bincode::serialize(self)
            .expect("Failed to serialize token instruction")
    }

    pub fn unpack(data: &[u8]) -> Result<Self, String> {
        bincode::deserialize(data)
            .map_err(|e| format!("Failed to deserialize token instruction: {}", e))
    }
}

/// Problem: Handle account data layout
///
/// Create a structure with a specific binary layout
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AccountDataLayout {
    pub is_initialized: u8,
    pub owner: [u8; 32],
    pub lamports: u64,
    pub data_length: u64,
    // Followed by dynamic data
}

impl AccountDataLayout {
    pub fn from_bytes(data: &[u8]) -> Result<(&Self, &[u8]), String> {
        if data.len() < std::mem::size_of::<Self>() {
            return Err(format!("Data too small for AccountDataLayout"));
        }

        let (header, remaining) = data.split_at(std::mem::size_of::<Self>());
        let header = unsafe { &*(header.as_ptr() as *const Self) };

        Ok((header, remaining))
    }
}

/// Problem: Create a Merkle tree for efficient verification
///
/// Implement a simple Merkle tree structure
pub struct MerkleTree {
    nodes: Vec<[u8; 32]>,
    leaf_count: usize,
}

impl MerkleTree {
    pub fn new(leaves: &[[u8; 32]]) -> Self {
        let leaf_count = leaves.len();
        let mut nodes = vec![[0; 32]; leaf_count * 2 - 1];

        // Copy the leaves
        for (i, leaf) in leaves.iter().enumerate() {
            nodes[i] = *leaf;
        }

        // Build the interior nodes
        for i in (0..leaf_count - 1).rev() {
            let left = nodes[2 * i + 1];
            let right = nodes[2 * i + 2];
            nodes[i] = Self::hash_pair(&left, &right);
        }

        MerkleTree {
            nodes,
            leaf_count,
        }
    }

    fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        let mut data = Vec::with_capacity(64);
        data.extend_from_slice(left);
        data.extend_from_slice(right);
        sha256_hash(&data)
    }

    pub fn root(&self) -> [u8; 32] {
        self.nodes[0]
    }

    pub fn generate_proof(&self, leaf_index: usize) -> Vec<[u8; 32]> {
        let mut proof = Vec::new();
        let mut index = leaf_index;

        while index > 0 {
            let sibling = if index % 2 == 0 { index - 1 } else { index + 1 };
            proof.push(self.nodes[sibling]);
            index = (index - 1) / 2;
        }

        proof
    }

    pub fn verify_proof(root: &[u8; 32], leaf: &[u8; 32], proof: &[[u8; 32]], leaf_index: usize) -> bool {
        let mut current = *leaf;
        let mut index = leaf_index;

        for &sibling in proof {
            current = if index % 2 == 0 {
                Self::hash_pair(&sibling, &current)
            } else {
                Self::hash_pair(&current, &sibling)
            };

            index = (index - 1) / 2;
        }

        current == *root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hash() {
        let data = b"hello world";
        let hash = sha256_hash(data);

        // Expected hash of "hello world"
        let expected = hex::decode("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9")
            .unwrap();
        let mut expected_bytes = [0u8; 32];
        expected_bytes.copy_from_slice(&expected);

        assert_eq!(hash, expected_bytes);
    }

    #[test]
    fn test_public_key() {
        // Create a test key
        let key_data = [42u8; 32];
        let key = PublicKey::from_bytes(&key_data).unwrap();

        // Test conversion methods
        assert_eq!(key.as_bytes(), &key_data);
        assert_eq!(PublicKey::try_from(key_data.as_ref()).unwrap(), key);

        // Test Base58
        let base58 = key.to_base58();
        assert!(!base58.is_empty());
    }

    #[test]
    fn test_keypair_generation_and_signing() {
        let message = b"Test message for signing";

        // Generate a keypair
        let (secret, public) = generate_keypair();

        // Sign the message
        let signature = sign_message(message, &secret).unwrap();

        // Verify the signature
        let is_valid = verify_signature(message, &signature, &public).unwrap();
        assert!(is_valid);

        // Test with wrong message
        let wrong_message = b"Wrong message";
        let is_invalid = verify_signature(wrong_message, &signature, &public).unwrap();
        assert!(!is_invalid);
    }

    #[test]
    fn test_token_account_serialization() {
        let owner = [1u8; 32];
        let mint = [2u8; 32];

        let account = TokenAccount::new(owner, mint);

        // Serialize
        let serialized = account.serialize().unwrap();

        // Deserialize
        let deserialized = TokenAccount::deserialize(&serialized).unwrap();

        // Verify
        assert_eq!(account, deserialized);
    }

    #[test]
    fn test_transaction() {
        let fee_payer = PublicKey([1u8; 32]);
        let blockhash = [3u8; 32];

        // Create a simple instruction
        let instruction = Instruction {
            program_id_index: 0,
            accounts: vec![1, 2],
            data: vec![0, 1, 2, 3],
        };

        // Account keys
        let account_keys = vec![[1u8; 32], [2u8; 32], [3u8; 32]];

        // Create transaction
        let mut tx = Transaction::new(
            &fee_payer,
            blockhash,
            vec![instruction],
            account_keys,
        );

        // Generate a keypair for signing
        let (secret, _) = generate_keypair();

        // Sign the transaction
        tx.sign(&secret).unwrap();

        // Serialize
        let serialized = tx.serialize().unwrap();

        // Deserialize
        let deserialized = Transaction::deserialize(&serialized).unwrap();

        // Verify signature count
        assert_eq!(deserialized.signatures.len(), 1);
    }

    #[test]
    fn test_token_instruction_packing() {
        let instruction = TokenInstruction::Transfer { amount: 1000 };

        // Pack
        let packed = instruction.pack();

        // Unpack
        let unpacked = TokenInstruction::unpack(&packed).unwrap();

        // Verify (we can't directly compare enums)
        match unpacked {
            TokenInstruction::Transfer { amount } => assert_eq!(amount, 1000),
            _ => panic!("Wrong instruction type unpacked"),
        }
    }

    #[test]
    fn test_account_data_layout() {
        // Create a test buffer
        let mut data = vec![0u8; 100];

        // Set some values
        data[0] = 1; // is_initialized
        data[1..33].copy_from_slice(&[5u8; 32]); // owner

        // Extract the header
        let (header, _) = AccountDataLayout::from_bytes(&data).unwrap();

        // Verify
        assert_eq!(header.is_initialized, 1);
        assert_eq!(header.owner, [5u8; 32]);
    }

    #[test]
    fn test_merkle_tree() {
        // Create some test leaves
        let leaves = vec![
            sha256_hash(b"leaf1"),
            sha256_hash(b"leaf2"),
            sha256_hash(b"leaf3"),
            sha256_hash(b"leaf4"),
        ];

        // Build the tree
        let tree = MerkleTree::new(&leaves);

        // Get root
        let root = tree.root();

        // Generate a proof for leaf 2
        let proof = tree.generate_proof(2);

        // Verify the proof
        let valid = MerkleTree::verify_proof(&root, &leaves[2], &proof, 2);
        assert!(valid);

        // Test with wrong leaf
        let invalid = MerkleTree::verify_proof(&root, &[0u8; 32], &proof, 2);
        assert!(!invalid);
    }
}
