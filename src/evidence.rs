//! Evidence bundle: a single self-contained JSON file that any third party
//! can verify offline with `xyon verify`.

use crate::keypair::Keypair;
use crate::ledger::{verify_events, Event, LedgerError, VerifyReport};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const BUNDLE_SCHEMA: &str = "xyon.evidence.v1";

#[derive(Serialize, Deserialize, Debug)]
pub struct EvidenceBundle {
    pub schema: String,
    pub sealed_at: String,
    pub principal: String,
    pub public_key: String,
    pub event_count: u64,
    pub head_hash: String,
    /// Signature by the principal over `head_hash|event_count` — the seal.
    pub seal_sig: String,
    pub events: Vec<Event>,
}

fn seal_bytes(head_hash: &str, event_count: u64) -> Vec<u8> {
    format!("xyon.seal.v1|{head_hash}|{event_count}").into_bytes()
}

/// Seal a ledger into an evidence bundle.
pub fn seal(events: Vec<Event>, keypair: &Keypair) -> Result<EvidenceBundle, LedgerError> {
    let pubkey_hex = keypair.public_key_hex();
    // Never seal a ledger we cannot ourselves verify.
    let report = verify_events(&events, &pubkey_hex)?;
    let seal_sig = hex::encode(keypair.sign(&seal_bytes(&report.head_hash, report.total)));
    Ok(EvidenceBundle {
        schema: BUNDLE_SCHEMA.to_string(),
        sealed_at: chrono::Utc::now().to_rfc3339(),
        principal: keypair.principal_id(),
        public_key: pubkey_hex,
        event_count: report.total,
        head_hash: report.head_hash,
        seal_sig,
        events,
    })
}

/// Verify a bundle read from disk. Checks, in order:
/// 1. every event signature and the hash chain,
/// 2. that the declared head hash / count match the recomputed chain,
/// 3. the seal signature over the head.
pub fn verify_bundle(path: &Path) -> Result<VerifyReport, LedgerError> {
    let raw = std::fs::read_to_string(path)?;
    let bundle: EvidenceBundle = serde_json::from_str(&raw)?;

    let report = verify_events(&bundle.events, &bundle.public_key)?;

    if report.head_hash != bundle.head_hash || report.total != bundle.event_count {
        return Err(LedgerError::ChainBroken(
            report.total,
            "bundle header does not match recomputed chain".into(),
        ));
    }

    let pubkey = hex::decode(&bundle.public_key)
        .map_err(|e| crate::keypair::KeyError::Invalid(format!("pubkey is not hex: {e}")))?;
    let sig = hex::decode(&bundle.seal_sig)
        .map_err(|_| LedgerError::BadSignature(report.total))?;
    crate::keypair::verify_signature(&pubkey, &seal_bytes(&bundle.head_hash, bundle.event_count), &sig)
        .map_err(|_| LedgerError::BadSignature(report.total))?;

    Ok(report)
}
