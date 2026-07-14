//! Ed25519 identity for signing ledger events.
//!
//! Design carried over from the Zaion prototype: principal id is
//! base58(sha256(public_key)), signatures are raw ed25519 over
//! canonical envelope bytes.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyError {
    #[error("invalid key material: {0}")]
    Invalid(String),
    #[error("signature verification failed")]
    VerificationFailed,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct Keypair {
    signing: SigningKey,
}

impl Keypair {
    pub fn generate() -> Self {
        Self {
            signing: SigningKey::generate(&mut OsRng),
        }
    }

    pub fn load(path: &Path) -> Result<Self, KeyError> {
        let hex_str = std::fs::read_to_string(path)?;
        let bytes = hex::decode(hex_str.trim())
            .map_err(|e| KeyError::Invalid(format!("key file is not hex: {e}")))?;
        let arr: [u8; 32] = bytes
            .as_slice()
            .try_into()
            .map_err(|_| KeyError::Invalid("expected 32 bytes".into()))?;
        Ok(Self {
            signing: SigningKey::from_bytes(&arr),
        })
    }

    pub fn save(&self, path: &Path) -> Result<(), KeyError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, hex::encode(self.signing.to_bytes()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }

    pub fn public_key_hex(&self) -> String {
        hex::encode(self.signing.verifying_key().to_bytes())
    }

    pub fn principal_id(&self) -> String {
        principal_from_pubkey(&self.signing.verifying_key().to_bytes())
    }

    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.signing.sign(message).to_bytes().to_vec()
    }
}

pub fn principal_from_pubkey(pubkey: &[u8]) -> String {
    bs58::encode(Sha256::digest(pubkey)).into_string()
}

pub fn verify_signature(pubkey: &[u8], message: &[u8], sig: &[u8]) -> Result<(), KeyError> {
    let key_arr: [u8; 32] = pubkey
        .try_into()
        .map_err(|_| KeyError::Invalid("expected 32-byte public key".into()))?;
    let verifying =
        VerifyingKey::from_bytes(&key_arr).map_err(|e| KeyError::Invalid(e.to_string()))?;
    let sig_arr: [u8; 64] = sig
        .try_into()
        .map_err(|_| KeyError::Invalid("expected 64-byte signature".into()))?;
    verifying
        .verify(message, &Signature::from_bytes(&sig_arr))
        .map_err(|_| KeyError::VerificationFailed)
}
