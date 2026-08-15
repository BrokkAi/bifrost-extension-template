# Dependency attribution

The application is original Apache-2.0 work and depends on published Rust crates recorded exactly in `Cargo.lock`.

The direct runtime dependency is `brokk-bifrost-runtime = 0.10.1`, part of Bifrost, licensed under Apache-2.0. It supplies the documented extension workspace, canonical observation and relation contracts, and reproducible run-manifest validator.

`serde`, `serde_json`, and `sha2` provide data-model serialization and content hashing. `clap` supplies the example CLI parser. The LSP example uses `lsp-server` and `lsp-types`; the MCP example uses the official Rust SDK `rmcp` and the `tokio` runtime. `tempfile` is used only by tests. Their transitive dependency set and license texts remain the responsibility of their respective copyright holders.

The final source-publication audit is checked in as [`audits/dependency-licenses.tsv`](audits/dependency-licenses.tsv). It records every workspace and registry package selected by `Cargo.lock`, its version, declared SPDX expression, and source. CI regenerates this inventory from `cargo metadata --locked` and rejects drift, missing license declarations, Git dependencies, path dependencies outside this workspace, and unexpected registry sources.

Before distributing binaries or publishing any package or release:

1. Re-run the inventory check on the exact distribution commit.
2. Verify every package's declared license and any required notices against its source archive and the form being distributed.
3. Retain the audit output and applicable license texts with the distribution evidence; do not infer binary-distribution compliance only from this source audit.

This document is attribution guidance, not legal advice.
