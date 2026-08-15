# Build a static-analysis tool on Bifrost

Bifrost supplies the analysis boundary; your application owns its inputs, interpretation, and user-facing protocol. A practical tool has two layers:

1. An analysis core opens an immutable workspace and uses `brokk-bifrost-runtime::extension` to map source-backed observations and request bounded semantic relations.
2. A thin adapter exposes that core through a CLI, Language Server Protocol (LSP), Model Context Protocol (MCP), web service, editor plugin, or another API.

This repository compiles three adapters against the same `analyze_workspace` function. None imports Bifrost's own CLI, MCP, LSP, analyzers, stores, or language modules.

## Analysis core

Start with the exact published Bifrost runtime package plus serialization crates for your extension-owned request and result models:

```toml
[dependencies]
brokk-bifrost-runtime = "=0.10.1"
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.145"
sha2 = "0.10.8"
```

Open a workspace with `ExtensionWorkspace::open`, select seeds from source spans, and make finite requests through `brokk_bifrost_runtime::extension`. The implementation in [`src/lib.rs`](../src/lib.rs) exposes the resulting protocol-neutral entry point:

```rust
let report = bifrost_extension_template::analyze_workspace(
    &bifrost_extension_template::AnalysisOptions {
        workspace: workspace.into(),
        config: config.into(),
        observations: observations.into(),
    },
)?;
```

The returned report keeps stable identities, proof, completeness, boundaries, generation, and digests. Serialize it freely; do not persist response-local dense node aliases or interpret an incomplete result as authoritative absence.

## CLI

The root binary adds `clap` for argument parsing:

```toml
clap = { version = "4.6.6", features = ["derive"] }
```

Run the general analysis command against any compatible workspace and extension-owned inputs:

```console
cargo run --locked -- analyze \
  --workspace fixtures/workspace \
  --config fixtures/input/config.json \
  --observations fixtures/input/observations.json
```

The command writes the shared analysis report as JSON to standard output. [`src/main.rs`](../src/main.rs) also retains the evidence-bundle lifecycle commands.

## LSP server

An LSP adapter needs the transport scaffold and protocol types; it depends on this repository only for the shared analysis core:

```toml
[dependencies]
bifrost-extension-template = { path = "../.." }
lsp-server = "0.10.0"
lsp-types = "0.97.0"
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.145"
```

[`adapters/lsp`](../adapters/lsp) is a standard-input/output language server. It advertises `workspace/executeCommand` and the command `bifrost-extension.analyze`. The first command argument is:

```json
{
  "workspace": "fixtures/workspace",
  "config": "fixtures/input/config.json",
  "observations": "fixtures/input/observations.json"
}
```

The LSP response result is exactly the protocol-neutral analysis report. A real editor extension can add document synchronization, diagnostics, code lenses, or custom commands around this seam without exposing Bifrost-private types.

Run the server with:

```console
cargo run --locked -p bifrost-extension-lsp-example
```

## MCP server

An MCP adapter adds the official Rust SDK and an async runtime:

```toml
[dependencies]
bifrost-extension-template = { path = "../.." }
rmcp = { version = "3.1.2", features = ["transport-io"] }
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.145"
tokio = { version = "1.48.0", features = ["macros", "rt-multi-thread"] }
```

[`adapters/mcp`](../adapters/mcp) exposes one typed `analyze_workspace` tool over standard input/output. Its three path arguments and structured result match the CLI and LSP forms. Run it with:

```console
cargo run --locked -p bifrost-extension-mcp-example
```

The MCP SDK generates the input and output schemas and handles MCP framing. The adapter owns the tool name and description; Bifrost remains behind the documented extension API.

## Choosing a surface

Use the CLI for scripts and CI, LSP when an editor needs request/response integration and diagnostics, and MCP when an agent or MCP client should discover and call the analysis tool. These are adapters, not separate analyzers: keep one tested analysis core and make each protocol layer responsible only for validation, transport, and error mapping.

For a separately published tool, replace the local `bifrost-extension-template` path dependency in the adapter examples with your own core crate. Keep Bifrost itself on an exact published version, run from a clean clone without a Bifrost checkout, and audit the final dependency and license graph before publishing.
