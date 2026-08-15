# Bifrost Extension Template

A reference application for building independent Rust and JSON/JSONL extensions on Bifrost's stable, bounded, evidence-carrying API.

This repository is the downstream proof for [Bifrost epic #2099](https://github.com/BrokkAi/bifrost/issues/2099). Its source is public for inspection and reuse under Apache-2.0, but it is not published as a crate, package, template listing, or release.

## What the example proves

The example performs the complete extension lifecycle without importing Bifrost implementation internals:

1. Open a fixture workspace as an immutable generation and record its capability report.
2. Resolve a unique source-backed seed from extension-owned configuration.
3. Adapt generic observation records into Bifrost's versioned observation document.
4. Map every record to a terminal exact, ambiguous, unmapped, stale, unsupported, or truncated outcome.
5. Request finite procedure-local control- and value-dependence relations.
6. Join exact observations to relation endpoints by stable identity, producing domain-neutral observed relation links.
7. Prove equivalent direct Rust, canonical JSON, and JSONL behavior.
8. Write and verify deterministic versioned artifacts and a #2105 run manifest.
9. Reopen unchanged source, confirm the immutable generation, and truthfully declare the second analyzer construction as `rebuilt`.

The derived result never persists response-local dense node numbers. Raw artifacts retain stable source identity, proof, completeness, boundaries, diagnostics, work, limits, generation, and provenance. An incomplete acquisition never becomes an authoritative absence claim.

## Build your own tool

The lifecycle is also a reusable static-analysis core. This repository includes three compile-tested ways to put an existing API in front of the same Bifrost-backed analysis:

- The root binary is a `clap` CLI that emits the protocol-neutral report as JSON.
- [`adapters/lsp`](adapters/lsp) is a standard LSP server exposing the analysis through `workspace/executeCommand`.
- [`adapters/mcp`](adapters/mcp) is an MCP server exposing a typed `analyze_workspace` tool over standard input/output.

Each adapter owns only transport, request validation, and error mapping. It does not depend on or expose Bifrost's own CLI, LSP, MCP, or implementation types. See [Build a static-analysis tool on Bifrost](docs/BUILDING_TOOLS.md) for the architecture, runnable commands, request examples, and exact Rust crates required for the analysis core, CLI, LSP, and MCP layers.

## Quick start

Install Rust 1.96 or newer, then run from the repository root:

```console
cargo run --locked -- run-example --output artifacts/example
cargo run --locked -- verify --bundle artifacts/example/cold
cargo run --locked -- verify --bundle artifacts/example/reopen
cargo run --locked -- reproduce --bundle artifacts/example/cold --workspace fixtures/workspace --output artifacts/reproduced
```

The first command prints the shared workspace generation and two manifest digests. The verification commands independently check canonical manifest encoding, component paths, sizes, hashes, dependency roles, completion consistency, and aggregate digest. The reproduction command consumes the verified bundle inputs, checks the exact workspace generation and source inventory, recreates the lifecycle, and requires the selected manifest digest to match. A relocated or changed workspace produces a precise prerequisite mismatch instead of a false reproduction claim.

The command refuses to overwrite an existing output path. Generated bundles belong under `artifacts/`, which is ignored by Git.

## Inputs and analysis

[`fixtures/input/config.json`](fixtures/input/config.json) is extension-owned configuration. It identifies the workspace revision, source seed, relation kinds, direction, and finite budgets. [`fixtures/input/observations.json`](fixtures/input/observations.json) is a tool-neutral example input whose records contain unique source snippets and caller-owned scalar attributes. The adapter resolves each snippet to a UTF-8 byte range and adds exact path/content identity before calling Bifrost.

The example analysis emits an `observed relation link` when an exactly mapped observation node is an endpoint of a bounded relation edge. Each link retains stable endpoint IDs, relation kind, proof, completeness, and source evidence. It computes no score, ordering, or domain-specific conclusion.

See [Artifact and evidence guide](docs/ARTIFACTS.md) for the bundle layout and completion rules.

## API and dependency boundary

`Cargo.toml` pins the published package exactly:

```toml
brokk-bifrost-runtime = "=0.10.1"
```

There is no path or Git dependency and no Bifrost source checkout is required. All integration goes through `brokk_bifrost_runtime::extension` or its canonical JSON/JSONL codecs. Bifrost never depends on this repository.

The template does not expose Bifrost's analyzers, stores, database schemas, arenas, solver plans, language modules, or protocol implementation types. Its adapters use the independent public `lsp-types` and `rmcp` crates, and all Bifrost integration stays within the documented extension surface.

Use `Path` and `PathBuf` for filesystem access. Protocol identities use canonical forward-slash relative paths because those bytes are platform-independent and content-addressed.

## Cache-state statement

The public 0.10.1 workspace API freezes an immutable source generation and builds an ephemeral analyzer on each open. The cold manifest therefore declares `fully_cold`; the same-process reopen manifest declares `rebuilt`, `warmup_count = 1`, and no persisted source or semantic artifact reuse. Do not change that declaration to a reuse claim unless a future documented API supplies evidence for it.

## Development and CI

Run the complete local gate:

```console
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

GitHub Actions repeats these checks, verifies the locked dependency/license inventory, and runs the lifecycle smoke test on Linux, macOS, and Windows. Third-party Actions are pinned to full commit hashes, and a lockfile-sensitive, Bifrost-versioned Rust cache reuses registry and build outputs without caching lifecycle evidence bundles. Superseded PR runs are cancelled automatically. Behavior tests cover positive, near-miss, unsupported, stale, truncated, cancelled, and incomplete outcomes. A reproducibility test compares every artifact byte across two fresh runs.

## Evidence categories and citation

A run manifest labels its purpose as one of:

- **Conformance evidence** checks a declared expectation against a comparison artifact.
- **Development experiments** explore behavior and may be incomplete; the bundled example uses this category.
- **Confirmatory results** require a preregistered or otherwise frozen protocol artifact.

A valid manifest proves artifact identity and declared acquisition state. It does not by itself establish that an experimental design or interpretation is sound.

When publishing results produced with Bifrost, cite the Bifrost version/revision, this extension and its immutable identity, the manifest digest, workspace revision, observation/configuration digests, limits, completion state, deviations, and cache declaration. See [`CITATION.cff`](CITATION.cff) and [Publication and citation guide](docs/PUBLISHING.md).

## Licensing and distribution

This repository and Bifrost are licensed under the Apache License 2.0. See [`LICENSE`](LICENSE). Dependencies retain their own licenses; see [Dependency attribution](DEPENDENCIES.md).

The source-publication gates and their evidence are recorded in the [Publication and citation guide](docs/PUBLISHING.md). `publish = false` is intentional: making the source repository public does not publish a crate, package, template listing, or release.
