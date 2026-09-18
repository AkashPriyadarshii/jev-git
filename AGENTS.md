# AGENTS.md — jev-git

This document guides AI coding agents working on `jev-git`.

## Architecture & Principles
- **Binary:** Single Rust binary exposing subcommands (`install`, `check`).
- **Git Alias:** Callable as `git-jev` or `git jev`.
- **Latency Budget:** Sub-100ms total wall time.
- **Engine:** TypeSafe AI Jev System One endpoint (`https://api.typesafe.ai/v1/systemone`).
- **Dependencies:** Minimalist (`ureq`, `serde`, `serde_json`). No async runtime.

## Verification
- Run tests: `cargo test`
- Build release: `cargo build --release`
