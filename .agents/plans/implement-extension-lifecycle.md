# Implement the complete Bifrost extension lifecycle proof

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must remain current. It follows the ExecPlan standard in Bifrost's `.agents/PLANS.md`: it is self-contained, uses observable milestones, and records exact commands and evidence so a new contributor can resume from this file alone.

## Purpose / Big Picture

This repository must prove that an independent Rust application can use only Bifrost's documented extension boundary to open an immutable workspace, map extension-owned observations, request finite semantic relations, run a small domain-neutral analysis, and emit reproducible evidence bundles. A contributor can see the lifecycle by running the single `run-example` command, inspecting two verified bundles, and observing that direct Rust, canonical JSON, and JSONL paths agree. The repository must not expose Bifrost implementation internals or claim that missing relations are authoritative when acquisition is incomplete.

## Progress

- [x] (2026-08-14 08:00Z) Fetched template `origin/main`, attached `dave/extension-lifecycle-proof`, and rebased onto `c3f9c355e7bf6b9d0edd16e7b23e2bd5a1c82d9b`.
- [x] (2026-08-14 08:20Z) Inspected Bifrost `origin/master` at `5a115bd8de56af1a35de71ca223d378c68cbc64f`, issues #2099-#2105, merged PRs #2113, #2116, #2117, #2119, #2120, #2121, #2125, and their seven ExecPlans.
- [x] (2026-08-14 08:30Z) Verified that published `brokk-bifrost-runtime` 0.9.5 contains the documented extension APIs and pinned it exactly.
- [x] (2026-08-14 09:05Z) Implemented the standalone application, fixtures, canonical JSON/JSONL evidence, stable-ID analysis, verified manifests, truthful reopen declaration, and reproduction preflight/execution.
- [x] (2026-08-14 09:25Z) Added positive, near-miss, unsupported, stale, truncated, cancelled, incomplete, deterministic, equivalence, bundle, and reproduction behavior tests.
- [x] (2026-08-14 09:40Z) Added private cross-platform CI, README, artifact/publication guides, citation metadata, dependency attribution, durable `AGENTS.md`, and explicit visibility/history gates.
- [x] (2026-08-14 10:18Z) Passed formatting, 10 integration tests, doctests, strict Clippy without suppressions, two-run byte comparison, direct/serialized/JSONL equivalence, generated-artifact inspection, and a registry-only clean-copy build/test/run with no Bifrost checkout.
- [x] (2026-08-14 10:25Z) Committed the scoped implementation without publishing or changing visibility.
- [x] (2026-08-14 10:30Z) Pushed `dave/extension-lifecycle-proof` and opened ready private PR #1 to `main` without changing visibility or publishing anything.
- [x] (2026-08-14 11:08Z) Observed private PR #1 CI run 31792059392 to completion: Ubuntu passed in 7m54s, macOS passed in 7m48s, and Windows passed in 24m39s. No platform-specific follow-up was required.
- [x] (2026-08-14 11:14Z) After PR #1 merged, replaced `main` with a content-preserving clean root history so the obsolete pre-Apache licensing statement is not reachable from the publication branch. Repository visibility remained private and no package or release was published.
- [x] (2026-08-15 08:01Z) Added compile-tested CLI, LSP, and MCP adapters around one protocol-neutral analysis entry point, documented the exact crates for each layer, and passed formatting, 11 tests, strict workspace Clippy, dependency-tree inspection, and a real CLI analysis run.
- [x] (2026-08-15) Added lockfile-sensitive Rust caching to the three-platform workflow and pinned checkout, toolchain, and cache Actions to verified full commit hashes; lifecycle evidence remains freshly generated and uncached.
- [x] (2026-08-15 14:39Z) Upgraded the exact published Bifrost runtime pin and all locked Bifrost crates to Apache-2.0 version 0.10.1. Formatting, 11 workspace tests, strict Clippy, dependency-tree inspection, and a clean-copy lifecycle build/test/run/verify/reproduce all passed without a Bifrost checkout.
- [x] (2026-08-15 15:35Z) Merged the 0.10.1 integration after Linux, macOS, and Windows CI passed, then replaced `main` with a content-identical clean root as authorized and confirmed only `main` remained on the remote.
- [x] (2026-08-15 16:05Z) Prepared the source-publication commit: retained `publish = false`, removed private-development wording, added a deterministic 297-record locked dependency/license inventory and CI drift check, and documented that public source visibility does not authorize any crate, package, template listing, release, or binary distribution. A no-Git isolated copy at `/private/tmp/bifrost-extension-public.3D0YKL` passed the locked workspace build, 11 tests, strict Clippy, lifecycle generation, cold/reopen verification, reproduction, and inventory check without a Bifrost checkout.

## Surprises & Discoveries

- Observation: The extension APIs merged after the original template foundation are already present in the crates.io 0.9.5 archive, so a Git dependency is unnecessary.
  Evidence: `cargo info brokk-bifrost-runtime@0.9.5` downloaded the package, and its `src/extension/` contains workspace, relation, observation, and artifact modules. `.cargo_vcs_info.json` records `a3ca30bd3fb994cc07db4abf47a2c796854882ca`.
- Observation: `ExtensionWorkspace::open` freezes source into an immutable overlay but constructs an ephemeral analyzer on every open; it exposes no claim of persistent or in-memory semantic cache reuse.
  Evidence: the public implementation calls `WorkspaceAnalyzer::build_ephemeral`. The reopen manifest must therefore declare `CacheStateKind::Rebuilt`, not a false reuse claim.
- Observation: Relocating byte-identical fixture source changes the workspace generation and generation-bound semantic node IDs, while canonical source paths and extension input digests remain unchanged.
  Evidence: the repository worktree generation was `084638d3...`; the clean-copy root generation was `f7108682...`. The `reproduce` command therefore checks generation and reports the expected/observed mismatch before execution rather than claiming cross-root equivalence.
- Observation: The first 0.10.1 Ubuntu CI run restored a 1.079 GB fallback target cache produced for the prior Bifrost lockfile, then exhausted runner disk while compiling a second dependency graph for the lifecycle binary.
  Evidence: CI run 31890567649 reported a non-full cache match, 98 MB free, and `rustc-LLVM ERROR: IO failure on output stream: No space left on device`. The cache namespace now includes the Bifrost version, and CI disables dev/test debug information to bound artifact size.

## Decision Log

- Decision: Use an exact registry dependency and no Bifrost source path or Git checkout; the current pin is `brokk-bifrost-runtime = "=0.10.1"`.
  Rationale: The published package is suitable, makes the dependency boundary independently testable, and satisfies the clean-consumer requirement.
  Date/Author: 2026-08-14 / Codex.
- Decision: The example analysis emits observed relation links instead of scores or rankings.
  Rationale: Joining exact observation stable IDs to bounded control/value relation endpoints is nontrivial and useful for impact exploration while remaining neutral about datasets, formulas, and research conclusions.
  Date/Author: 2026-08-14 / Codex.
- Decision: Preserve raw observation and relation artifacts beside the derived result, and permit authoritative absence only when both acquisitions are complete.
  Rationale: A compact derived artifact alone would lose proof, completeness, boundaries, diagnostics, work, limits, generation, and provenance.
  Date/Author: 2026-08-14 / Codex.
- Decision: Represent the second open as a same-process rebuild with one warmup, not as cache reuse.
  Rationale: Cache-state declarations are evidence, not aspiration; the current API does not expose reusable workspace state.
  Date/Author: 2026-08-14 / Codex.
- Decision: Keep CLI, LSP, and MCP as thin adapters over `analyze_workspace` rather than importing Bifrost's own protocol implementations.
  Rationale: An extension author should own the public tool contract while Bifrost remains behind its documented extension boundary; one shared result also makes protocol equivalence straightforward to test.
  Date/Author: 2026-08-15 / Codex.
- Decision: Cache Cargo registry and compilation outputs in CI, but never cache generated lifecycle bundles.
  Rationale: Dependency compilation is the dominant repeat cost, while cold/reopen bundle generation and reproduction must continue to exercise fresh evidence production. The cache namespace includes the Bifrost version to prevent cross-version target fallback from exhausting runner disk. Full Action commit pins make workflow execution immutable even though comments retain recognizable release-family labels.
  Date/Author: 2026-08-15 / Codex.

## Outcomes & Retrospective

The downstream proof and source-publication preparation are complete. It opens immutable workspaces, adapts generic observations, requests finite source-backed control/value relations, joins stable identities without persisting dense aliases, preserves raw evidence and incomplete boundaries, emits deterministic #2105 bundles, and recreates or precisely rejects reproduction prerequisites. A reusable `analyze_workspace` boundary drives a JSON CLI, a standard LSP `workspace/executeCommand`, and a typed MCP tool without importing Bifrost protocol implementations. Local validation, an isolated registry-only copy, and Linux/macOS/Windows CI pass. After the implementation merged, `main` was rewritten to a clean root so obsolete licensing language is not reachable from the publication branch. The source-publication commit adds a checked, deterministic inventory of every locked package and license declaration. The package remains non-publishable; any crate, package, template listing, release, or binary distribution requires separate review and authorization.

## Context and Orientation

`src/lib.rs` owns the complete lifecycle and exposes testable functions. `src/main.rs` is a small command-line wrapper. `fixtures/workspace/` is a tiny TypeScript program; `fixtures/input/` contains extension-owned configuration and generic observation records expressed by unique source snippets rather than analyzer IDs. The application converts those records into Bifrost's canonical observation document, asks for a procedure-bounded semantic snapshot, and joins only stable IDs. `tests/` covers the lifecycle and adverse behavior. `.github/workflows/ci.yml` runs the same checks on three operating systems.

`adapters/lsp/` and `adapters/mcp/` are independent workspace packages that translate standard protocol requests into `AnalysisOptions`. `docs/BUILDING_TOOLS.md` identifies the exact analysis, CLI, LSP, and MCP dependencies and explains when to use each surface.

A workspace generation is Bifrost's immutable digest of frozen source and configuration. A stable node ID is a content-derived identity that may be persisted; a dense local ID is only a small number inside one snapshot and must never be persisted as identity. A boundary is a typed explanation of missing or partial semantic acquisition. A run bundle is a directory containing canonical component files and `manifest.json`, whose component hashes and aggregate digest are validated by Bifrost's #2105 contract.

## Plan of Work

Create an Apache-2.0 Rust package that is explicitly non-publishable and pins the published runtime. Implement strict deserialization for configuration and observation inputs. Resolve source snippets uniquely with UTF-8 byte offsets and construct canonical Bifrost identities, limits, and provenance. Execute observation mapping and semantic relation requests both directly and through the transport-neutral request envelope; serialize and read back JSONL results; reject any semantic difference.

Derive a sorted result whose links retain record ID, endpoint stable IDs, relation kind, proof, completeness, and evidence mappings. Include raw canonical inputs and outputs, capabilities, equivalence evidence, and the derived result as manifest components. Generate a fully-cold bundle and then reopen the unchanged workspace to generate a same-process rebuilt bundle with the same generation and deterministic semantic results. Verify both bundles after writing them.

Add tests for positive execution, a near-miss observation, unsupported source language, stale content/generation, tiny-limit truncation, and incomplete absence semantics. Add deterministic two-directory byte comparison and direct-versus-serialized assertions. Document local use, schema boundaries, citation, publication gates, and current Apache-2.0 dependency attribution. Add a cross-platform private Actions workflow.

## Concrete Steps

From the repository root, run:

    cargo run --locked -- run-example --output artifacts/example
    cargo run --locked -- verify --bundle artifacts/example/cold
    cargo run --locked -- verify --bundle artifacts/example/reopen
    cargo run --locked -- reproduce --bundle artifacts/example/cold --workspace fixtures/workspace --output artifacts/reproduced

The first command must report one cold and one rebuilt manifest digest and their shared generation. The verification commands must report `verified` and the manifest digest. Generated output stays ignored.

For validation, run:

    cargo fmt --check
    cargo test --workspace
    cargo clippy --workspace --all-targets -- -D warnings

Then copy the repository without `.git` and without any sibling Bifrost checkout into a temporary directory and rerun the locked build and tests. `cargo tree` must show `brokk-bifrost-runtime v0.10.1` from the registry and no path dependency.

## Validation and Acceptance

Acceptance requires all tests and Clippy to pass with no lint suppression. A fresh-run test must create two independent output roots and compare every relative file byte-for-byte. The direct/serialized test must compare canonical Bifrost responses and JSONL round trips. Adverse tests must prove that stale and unsupported states remain typed, truncation remains incomplete, near-miss records are terminally unmapped rather than silently absent, and `authoritative_absence` is false whenever either acquisition is incomplete.

The README and citation file must render as readable Markdown/YAML, use platform-neutral commands and paths, distinguish public source visibility from non-published package status, distinguish conformance/development/confirmatory evidence, and avoid excluded research vocabulary and Bifrost-private type names except in the explicit boundary warning.

## Idempotence and Recovery

Build and test commands are safe to repeat. The application refuses to overwrite an existing output directory; choose a fresh directory or remove only a known generated directory under `artifacts/`. It writes complete bundle contents before validation. If interrupted, remove only the reported generated directory and rerun. Never remove a repository root or user-supplied workspace.

## Artifacts and Notes

The authoritative source package revision for `brokk-bifrost-runtime` 0.10.1 is `511adaa2733067bb1b7809ab79e06ec0e3d2a146`. The Bifrost master inspected for the original contract context was `5a115bd8de56af1a35de71ca223d378c68cbc64f`. These are intentionally distinct: the template executes the published package, not an arbitrary source checkout.

Validation evidence at completion:

    cargo fmt --check
    cargo test --locked --workspace
    test result: ok. 6 behavior + 4 lifecycle; 0 failed
    cargo clippy --locked --workspace --all-targets -- -D warnings
    Finished `dev` profile ...

The isolated copy `/private/tmp/bifrost-extension-clean.IsrvlY` contained no `.git` metadata or Bifrost checkout. `cargo tree --locked` resolved `brokk-bifrost-runtime v0.9.5` from the registry; a from-empty-target build took 3m18s and all tests plus the lifecycle passed. Generated cold/reopen artifacts were verified and `CITATION.cff` parsed as YAML with Apache-2.0.

## Interfaces and Dependencies

`src/lib.rs` exposes `run_lifecycle(&RunOptions)`, `verify_bundle(&Path)`, and `reproduce_bundle(bundle, workspace, output)`. The command line supports `run-example`, `verify`, and `reproduce`. All Bifrost interaction goes through explicitly imported documented items from `brokk_bifrost_runtime::extension`. `serde`, `serde_json`, and `sha2` provide strict input models and deterministic hashing; `tempfile` is test-only.

Revision note (2026-08-14): Created the self-contained plan after live inspection. It records the published-package discovery, truthful rebuild cache declaration, and public-history gate because those materially changed the initial implementation assumptions.

Revision note (2026-08-14 10:18Z): Marked implementation and validation complete, added reproduction behavior and clean-copy evidence, and recorded generation-bound relocation behavior so future contributors do not claim cross-root identity equivalence.

Revision note (2026-08-14 10:25Z): Recorded the implementation commit and narrowed remaining work to push, ready-PR creation, and remote CI observation.

Revision note (2026-08-14 10:30Z): Recorded the private branch push and ready PR #1; only remote Linux/macOS/Windows CI observation remains.

Revision note (2026-08-14 11:08Z): Recorded successful completion of all three private CI jobs. The implementation plan now has no remaining private-development work; publication gates intentionally remain external future actions.

Revision note (2026-08-14 11:14Z): Recorded the maintainer-authorized post-merge `main` history replacement. The publication audit remains a gate so future commits and refs are rechecked immediately before visibility changes.

Revision note (2026-08-15 08:01Z): Added the shared analysis entry point and compile-tested CLI, LSP, and MCP adapters, plus exact dependency and usage guidance. The adapters preserve the extension boundary by translating public protocol requests into extension-owned inputs and serializing the same stable report.

Revision note (2026-08-15 14:39Z): Upgraded the registry-only Bifrost dependency and complete locked Bifrost graph to 0.10.1, recording package revision `511adaa2733067bb1b7809ab79e06ec0e3d2a146`. The documented extension module remained source-identical, and both the workspace gate and an isolated no-Git clean-copy lifecycle passed.

Revision note (2026-08-15 16:05Z): Recorded the merged 0.10.1 validation and source-publication preparation. Added a checked locked-graph license inventory while preserving `publish = false` and the explicit prohibition on package, template, release, and binary publication.
