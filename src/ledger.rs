//! Append-only, hash-chained, signed event ledger stored as JSONL.
//!
//! Each line is one event. Chain integrity: every event embeds the hash of
//! the previous event; the first event links to a genesis hash of all zeros.
//! Every event is ed25519-signed by the recording principal, so a verifier
//! needs nothing but this file and the public key to check both integrity
//! (no line inserted/removed/reordered) and authenticity (every line was
//! produced by the keyholder).

use crate::keypair::{verify_signature, Keypair};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{BufRead, Write};
use std::path::Path;
use thiserror::Error;

pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
pub const EVENT_SCHEMA: &str = "xyon.event.v1";

#[derive(Error, Debug)]
pub enum LedgerError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("chain broken at seq {0}: {1}")]
    ChainBroken(u64, String),
    #[error("bad signature at seq {0}")]
    BadSignature(u64),
    #[error("key error: {0}")]
    Key(#[from] crate::keypair::KeyError),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Event {
    pub schema: String,
    pub seq: u64,
    pub prev: String,
    pub ts: String,
    pub principal: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub payload: serde_json::Value,
    pub sig: String,
}

impl Event {
    /// Canonical bytes covered by the signature: the event with `sig` empty.
    fn signing_bytes(&self) -> Vec<u8> {
        let unsigned = serde_json::json!({
            "schema": self.schema,
            "seq": self.seq,
            "prev": self.prev,
            "ts": self.ts,
            "principal": self.principal,
            "type": self.event_type,
            "payload": self.payload,
        });
        serde_json::to_vec(&unsigned).expect("Value serialization is infallible")
    }

    /// Chain hash of this event: sha256(signing_bytes || sig_hex).
    pub fn hash(&self) -> String {
        let mut h = Sha256::new();
        h.update(self.signing_bytes());
        h.update(self.sig.as_bytes());
        hex::encode(h.finalize())
    }
}

pub struct Ledger {
    path: std::path::PathBuf,
}

#[derive(Debug)]
pub struct VerifyReport {
    pub total: u64,
    pub principal: Option<String>,
    pub head_hash: String,
}

impl Ledger {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Read the last event, if any (streams the file; ledgers are per-mission,
    /// so linear scan is fine at this scale).
    fn tail(&self) -> Result<Option<Event>, LedgerError> {
        if !self.path.exists() {
            return Ok(None);
        }
        let file = std::fs::File::open(&self.path)?;
        let mut last: Option<Event> = None;
        for line in std::io::BufReader::new(file).lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            last = Some(serde_json::from_str(&line)?);
        }
        Ok(last)
    }

    /// Append a signed event and return it.
    pub fn append(
        &self,
        keypair: &Keypair,
        event_type: &str,
        payload: serde_json::Value,
    ) -> Result<Event, LedgerError> {
        let (seq, prev) = match self.tail()? {
            None => (0, GENESIS_HASH.to_string()),
            Some(prev_event) => (prev_event.seq + 1, prev_event.hash()),
        };
        let mut event = Event {
            schema: EVENT_SCHEMA.to_string(),
            seq,
            prev,
            ts: chrono::Utc::now().to_rfc3339(),
            principal: keypair.principal_id(),
            event_type: event_type.to_string(),
            payload,
            sig: String::new(),
        };
        event.sig = hex::encode(keypair.sign(&event.signing_bytes()));

        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        serde_json::to_writer(&mut file, &event)?;
        file.write_all(b"\n")?;
        Ok(event)
    }

    pub fn read_all(&self) -> Result<Vec<Event>, LedgerError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let file = std::fs::File::open(&self.path)?;
        let mut events = Vec::new();
        for line in std::io::BufReader::new(file).lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            events.push(serde_json::from_str(&line)?);
        }
        Ok(events)
    }
}

/// Verify chain integrity and every signature for a slice of events.
///
/// `pubkey_hex` is the recording principal's public key. Fails on the first
/// broken link, out-of-order seq, principal mismatch, or bad signature.
pub fn verify_events(events: &[Event], pubkey_hex: &str) -> Result<VerifyReport, LedgerError> {
    let pubkey = hex::decode(pubkey_hex)
        .map_err(|e| crate::keypair::KeyError::Invalid(format!("pubkey is not hex: {e}")))?;
    let expected_principal = crate::keypair::principal_from_pubkey(&pubkey);

    let mut expected_prev = GENESIS_HASH.to_string();
    let mut head_hash = GENESIS_HASH.to_string();

    for (i, event) in events.iter().enumerate() {
        if event.seq != i as u64 {
            return Err(LedgerError::ChainBroken(
                event.seq,
                format!("expected seq {i}"),
            ));
        }
        if event.prev != expected_prev {
            return Err(LedgerError::ChainBroken(event.seq, "prev hash mismatch".into()));
        }
        if event.principal != expected_principal {
            return Err(LedgerError::ChainBroken(
                event.seq,
                "principal does not match public key".into(),
            ));
        }
        let sig = hex::decode(&event.sig)
            .map_err(|_| LedgerError::BadSignature(event.seq))?;
        verify_signature(&pubkey, &event.signing_bytes(), &sig)
            .map_err(|_| LedgerError::BadSignature(event.seq))?;
        head_hash = event.hash();
        expected_prev = head_hash.clone();
    }

    Ok(VerifyReport {
        total: events.len() as u64,
        principal: events.first().map(|e| e.principal.clone()),
        head_hash,
    })
}
