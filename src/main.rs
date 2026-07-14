//! xyon — signed, tamper-evident evidence for AI agent actions.
//!
//! v0.0.1 surface, four commands:
//!   xyon init                 create an identity (~/.xyon/key)
//!   xyon record <type> [json] append a signed event (reads stdin if json is `-`)
//!   xyon seal <out.json>      seal the current ledger into an evidence bundle
//!   xyon verify <bundle.json> independently verify a bundle (needs no identity)

mod evidence;
mod keypair;
mod ledger;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use keypair::Keypair;
use ledger::Ledger;
use std::io::Read;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "xyon", version, about = "Signed evidence for AI agent actions")]
struct Cli {
    /// Data directory (default: $XYON_HOME or ~/.xyon)
    #[arg(long, global = true)]
    home: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new identity keypair
    Init,
    /// Append a signed event to the active ledger
    Record {
        /// Event type, e.g. tool_call, approval, task_start
        event_type: String,
        /// JSON payload; use `-` to read from stdin
        payload: Option<String>,
    },
    /// Seal the active ledger into a self-contained evidence bundle
    Seal {
        /// Output path for the bundle
        out: PathBuf,
    },
    /// Verify an evidence bundle (requires no identity or network)
    Verify {
        /// Path to a bundle produced by `xyon seal`
        bundle: PathBuf,
    },
    /// Print the active ledger as human-readable lines
    Log,
}

fn home_dir(cli_home: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(h) = cli_home {
        return Ok(h);
    }
    if let Ok(env_home) = std::env::var("XYON_HOME") {
        return Ok(PathBuf::from(env_home));
    }
    let base = dirs_home().context("cannot determine home directory")?;
    Ok(base.join(".xyon"))
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let home = home_dir(cli.home)?;
    let key_path = home.join("key");
    let ledger = Ledger::new(home.join("ledger.jsonl"));

    match cli.command {
        Command::Init => {
            if key_path.exists() {
                bail!("identity already exists at {}", key_path.display());
            }
            let kp = Keypair::generate();
            kp.save(&key_path)?;
            println!("principal:  {}", kp.principal_id());
            println!("public key: {}", kp.public_key_hex());
            println!("key stored: {}", key_path.display());
        }
        Command::Record {
            event_type,
            payload,
        } => {
            let kp = Keypair::load(&key_path)
                .with_context(|| format!("no identity at {} — run `xyon init`", key_path.display()))?;
            let raw = match payload.as_deref() {
                None => "{}".to_string(),
                Some("-") => {
                    let mut buf = String::new();
                    std::io::stdin().read_to_string(&mut buf)?;
                    buf
                }
                Some(s) => s.to_string(),
            };
            let value: serde_json::Value =
                serde_json::from_str(&raw).context("payload is not valid JSON")?;
            let event = ledger.append(&kp, &event_type, value)?;
            println!("recorded seq={} type={}", event.seq, event.event_type);
        }
        Command::Seal { out } => {
            let kp = Keypair::load(&key_path)
                .with_context(|| format!("no identity at {} — run `xyon init`", key_path.display()))?;
            let events = ledger.read_all()?;
            if events.is_empty() {
                bail!("ledger is empty — nothing to seal");
            }
            let bundle = evidence::seal(events, &kp)?;
            std::fs::write(&out, serde_json::to_string_pretty(&bundle)?)?;
            println!(
                "sealed {} events · head {} · {}",
                bundle.event_count,
                &bundle.head_hash[..12],
                out.display()
            );
        }
        Command::Verify { bundle } => {
            let report = evidence::verify_bundle(&bundle)?;
            println!("✓ signature valid · chain intact");
            println!("  events:    {}", report.total);
            println!("  principal: {}", report.principal.unwrap_or_default());
            println!("  head:      {}", report.head_hash);
        }
        Command::Log => {
            for e in ledger.read_all()? {
                println!("{:>4}  {}  {:<12} {}", e.seq, e.ts, e.event_type, e.payload);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_home() -> tempfile::TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    #[test]
    fn record_seal_verify_roundtrip() {
        let home = temp_home();
        let key_path = home.path().join("key");
        let kp = Keypair::generate();
        kp.save(&key_path).unwrap();

        let ledger = Ledger::new(home.path().join("ledger.jsonl"));
        for i in 0..5 {
            ledger
                .append(&kp, "tool_call", serde_json::json!({ "n": i }))
                .unwrap();
        }

        let bundle = evidence::seal(ledger.read_all().unwrap(), &kp).unwrap();
        let out = home.path().join("evidence.json");
        std::fs::write(&out, serde_json::to_string(&bundle).unwrap()).unwrap();

        let report = evidence::verify_bundle(&out).unwrap();
        assert_eq!(report.total, 5);
    }

    #[test]
    fn tampered_payload_fails_verification() {
        let home = temp_home();
        let kp = Keypair::generate();
        let ledger = Ledger::new(home.path().join("ledger.jsonl"));
        ledger
            .append(&kp, "tool_call", serde_json::json!({ "cmd": "ls" }))
            .unwrap();
        ledger
            .append(&kp, "tool_call", serde_json::json!({ "cmd": "rm -rf /" }))
            .unwrap();

        let mut bundle = evidence::seal(ledger.read_all().unwrap(), &kp).unwrap();
        // Attacker rewrites history: soften the second command.
        bundle.events[1].payload = serde_json::json!({ "cmd": "ls -la" });
        let out = home.path().join("evidence.json");
        std::fs::write(&out, serde_json::to_string(&bundle).unwrap()).unwrap();

        assert!(evidence::verify_bundle(&out).is_err());
    }

    #[test]
    fn removed_event_breaks_chain() {
        let home = temp_home();
        let kp = Keypair::generate();
        let ledger = Ledger::new(home.path().join("ledger.jsonl"));
        for i in 0..3 {
            ledger
                .append(&kp, "step", serde_json::json!({ "n": i }))
                .unwrap();
        }
        let mut bundle = evidence::seal(ledger.read_all().unwrap(), &kp).unwrap();
        // Attacker deletes the middle event.
        bundle.events.remove(1);
        let out = home.path().join("evidence.json");
        std::fs::write(&out, serde_json::to_string(&bundle).unwrap()).unwrap();

        assert!(evidence::verify_bundle(&out).is_err());
    }

    #[test]
    fn foreign_key_cannot_seal_for_principal() {
        let home = temp_home();
        let kp = Keypair::generate();
        let ledger = Ledger::new(home.path().join("ledger.jsonl"));
        ledger
            .append(&kp, "step", serde_json::json!({}))
            .unwrap();

        // A different keypair tries to seal someone else's ledger.
        let imposter = Keypair::generate();
        assert!(evidence::seal(ledger.read_all().unwrap(), &imposter).is_err());
    }
}
