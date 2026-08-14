# Repository instructions

This repository is the downstream proof for Bifrost's documented extension API. Keep it independent: depend only on the published `brokk-bifrost-runtime::extension` surface or its canonical JSON/JSONL contracts. Do not import Bifrost analyzers, language modules, stores, SQLite schemas, arenas, solver plans, MCP, or LSP types, and do not copy Bifrost or Joern implementation code.

Keep all paths platform-independent with `Path` and `PathBuf`. Treat dense semantic node numbers as response-local aliases; persist and join only stable IDs plus their source spans and call context. Never turn an incomplete observation mapping or semantic snapshot into an authoritative absence claim.

Before committing, run:

    cargo fmt --check
    cargo test --workspace
    cargo clippy --workspace --all-targets -- -D warnings

Generated bundles belong under `artifacts/` and remain untracked. This repository is private and `publish = false`; do not publish a crate or release from this repository. Public-readiness requires the exact published Bifrost pin in `Cargo.toml`, a clean-clone build with no Bifrost source checkout, cross-platform CI, reproducibility checks, and a final dependency/license audit.

Maintain `.agents/plans/implement-extension-lifecycle.md` as a living ExecPlan whenever the lifecycle or its validation changes.
