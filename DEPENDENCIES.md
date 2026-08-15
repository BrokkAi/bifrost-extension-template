# Dependency attribution

The application is original Apache-2.0 work and depends on published Rust crates recorded exactly in `Cargo.lock`.

The direct runtime dependency is `brokk-bifrost-runtime = 0.10.1`, part of Bifrost, licensed under Apache-2.0. It supplies the documented extension workspace, canonical observation and relation contracts, and reproducible run-manifest validator.

`serde`, `serde_json`, and `sha2` provide data-model serialization and content hashing. `clap` supplies the example CLI parser. The LSP example uses `lsp-server` and `lsp-types`; the MCP example uses the official Rust SDK `rmcp` and the `tokio` runtime. `tempfile` is used only by tests. Their transitive dependency set and license texts remain the responsibility of their respective copyright holders.

Before any public distribution:

1. Generate a dependency inventory from the locked graph on the release commit.
2. Verify every package's declared license and any required notices against its source archive.
3. Confirm that the inventory contains only registry dependencies expected from `Cargo.lock` and no local path or Git dependency.
4. Retain the audit output with the release evidence; do not infer compliance only from this summary.

This document is attribution guidance, not legal advice.
