# xyon

Signed, tamper-evident evidence for AI agent actions.

xyon is a single small binary that records what an agent did into an
append-only, hash-chained, ed25519-signed ledger — and seals it into an
evidence bundle that **anyone can verify offline**, with no account, no
network, and no trust in the person handing it over.

## Status: v0.0.1 — honest and tiny

What exists today (everything below is implemented and tested):

- `xyon init` — create a local ed25519 identity
- `xyon record <type> [json]` — append a signed event to the ledger
- `xyon log` — print the ledger
- `xyon seal <out.json>` — seal the ledger into a self-contained bundle
- `xyon verify <bundle.json>` — independently verify chain + signatures

What does not exist yet: agent integration (Claude Code hooks), approval
gates, rollback, the `ci-guardian` employee described in [CHARTER.md](CHARTER.md).
This README will only ever describe things that are real.

## Install

There are no prebuilt releases yet. Build from source:

```bash
git clone https://github.com/zaimouren1/xyon
cd xyon
cargo build --release   # requires Rust 1.75+
```

## Try it in 60 seconds

```bash
export XYON_HOME=$(mktemp -d)   # keep the demo isolated
xyon init
xyon record task_start '{"mission":"demo"}'
xyon record tool_call  '{"tool":"bash","cmd":"cargo test"}'
xyon record task_end   '{"result":"pass"}'
xyon seal evidence.json
xyon verify evidence.json
# ✓ signature valid · chain intact
```

Now tamper with any byte of any event in `evidence.json` and run
`xyon verify` again — it fails. Delete an event — it fails. Reorder —
it fails. That is the entire point.

## Why

Agents are becoming capable faster than they are becoming trustworthy.
The missing piece is not intelligence — it is *evidence*: a record of what
an agent actually did that the agent (or its vendor, or its operator)
cannot quietly rewrite. xyon is the smallest possible version of that
record, built first, so everything later can stand on it.

The longer-term thesis lives in [CHARTER.md](CHARTER.md): agents with
verifiable track records — employees, not tools.

## Design

- One ledger = one JSONL file. Each line is one event, signed over its
  canonical bytes, carrying the hash of the previous event.
- A bundle = the ledger + a seal signature over the chain head.
- The verifier needs only the bundle. Key material never leaves `$XYON_HOME`.

## License

Apache-2.0
