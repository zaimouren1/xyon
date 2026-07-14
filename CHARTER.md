# CHARTER — the first AI employee of xyon

This file is the founding experiment of this repository. It is a mandate,
not a plan: it names one AI agent, one responsibility, one budget, and one
falsifiable success criterion, then runs for eight weeks.

## Appointment

- **Employee**: `ci-guardian` — an AI agent built on the Claude Agent SDK.
- **Employer**: the human owner of this repository.
- **Start**: the day the first `ci-guardian` evidence bundle is committed.
- **Term**: 8 weeks. Renewal requires the verdict below.

## Responsibility

Keep the CI of `xyon` healthy.

- CI green: stay silent.
- CI red: investigate, and either open a fix (every change goes through
  human approval) or escalate with a diagnosis.
- Weekly: file one signed work report as an evidence bundle in `reports/`.

## Authority and limits

- May: read the repo, run builds and tests, propose commits on branches.
- May not: push to `main`, touch secrets, spend beyond budget, or act
  outside this repository.
- Every tool call is recorded in a xyon ledger; every completed task is
  sealed with `xyon seal` and verifiable by anyone with `xyon verify`.

## Budget

- API cost: capped per month (set by employer at hire time).
- Human attention: escalations must include a diagnosis, not a question.

## The falsifiable verdict (week 8)

The experiment succeeds only if the week-8 employee is **measurably more
competent than the week-1 employee at this specific job**, shown by its
own signed evidence: faster diagnosis, fewer false escalations, or reports
that cite lessons learned in earlier weeks.

If the memory does not compound, the thesis of this project — that agents
can accrue verifiable track records worth trusting — fails cheaply, in
eight weeks, and this file will say so rather than be deleted.

## Status

- [ ] Employee not yet hired. Ledger and verifier shipped first (v0.0.1).
