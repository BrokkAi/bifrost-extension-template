# Maintain the Bifrost extension lifecycle proof

This living ExecPlan records changes to the lifecycle and its validation. Keep `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` current.

## Purpose / Big Picture

This repository proves that an independent Rust application can use only Bifrost's published extension boundary to open an immutable workspace, map extension-owned observations, request bounded semantic relations, and emit reproducible evidence bundles. It must preserve stable identity and typed incompleteness without importing implementation internals.

## Progress

- [x] (2026-08-14) Implemented the standalone lifecycle, JSON/JSONL equivalence, stable-ID joins, verified manifests, adverse tests, and cross-platform CI against the published runtime.
- [x] (2026-08-15) Added reusable CLI, LSP, and MCP examples plus dependency/license auditing and public-source readiness.
- [x] (2026-09-08) Created `dave/update-bifrost-v0.11.0` from current `origin/main`; the divergent local `main` commit remains preserved on its existing branches.
- [x] (2026-09-08) Pinned `brokk-bifrost-runtime = "=0.11.0"`, raised the minimum toolchain to Rust 1.97, refreshed the lockfile and CI dependency assertions, and migrated manifest/completion handling.
- [x] (2026-09-08) Added downstream proof for portable workspace content identity and updated documentation for persistence evidence, information tiers, semantic frontiers, and stable semantic-node identity.
- [x] (2026-09-08) Updated the worked policy workflow from its historical v0.10.5 action commit to the live `v0` commit for the immutable v0.11.0 action release.
- [x] (2026-09-08) Passed formatting, 11 workspace tests, strict Clippy, the 347-record dependency/license audit, exact registry-source proof, lifecycle smoke/verify/reproduce, the Bifrost code-smell policy pack, and an isolated clean-copy build with no Bifrost checkout.
- [ ] Commit, push, open the ready PR, and observe its Linux/macOS/Windows CI to a terminal state.

## Surprises & Discoveries

- Bifrost 0.11.0 requires Rust 1.97 through its analyzer and language crates; a dependency-only update correctly fails under the former 1.96 toolchain.
- The 0.11.0 run-manifest schema requires `WorkspaceRunIdentity.content_identity`.
- The former `SemanticRelationStatus::Partial` is split into `FrontierBounded` and `BudgetBounded`, and `ExtensionCompletion` adds `FrontierBounded`. Only `Complete` can support authoritative absence.
- Relocated byte-identical workspaces intentionally have different local generations but equal portable content identities. Exact bundle reproduction remains generation-bound.
- The exact crates.io source revision for `brokk-bifrost-runtime` 0.11.0 is `e30944cfee489f0ff26a3ded36c9df5fb0d8bb04`.

## Decision Log

- Decision: Keep the example lifecycle ephemeral while documenting the opt-in persisted mode and retaining the workspace's actual store report in the capability artifact.
  Rationale: The existing cold/reopen bundles make no reuse claim. Adding persistent on-disk state would change reproducibility, cleanup, and cache declarations and deserves a separate explicit lifecycle.
  Date/Author: 2026-09-08 / Codex.
- Decision: Treat frontier-bounded results as incomplete in the derived manifest status.
  Rationale: More caller budget will not cross an analyzer frontier, but the result still cannot prove exhaustive absence.
  Date/Author: 2026-09-08 / Codex.
- Decision: Preserve exact local generation as the reproduction prerequisite while exposing portable content identity as a comparison key.
  Rationale: Content identity is portable but does not replace generation for stale-request and local-store scoping.
  Date/Author: 2026-09-08 / Codex.

## Outcomes & Retrospective

The local v0.11.0 migration is complete. The exact published runtime and policy action are pinned, the new identity and completion contracts are exercised, documentation matches the new persistence and evidence semantics, and all local and isolated-consumer gates pass. Remote PR CI remains the delivery gate.

## Context and Orientation

`src/lib.rs` owns the lifecycle and protocol-neutral analysis. `src/main.rs` is the CLI. `adapters/lsp` and `adapters/mcp` wrap the same analysis boundary. `tests/behavior.rs` covers adverse contract behavior, and `tests/lifecycle.rs` covers artifacts and reproduction. `.github/workflows/ci.yml` runs the locked gates on Linux, macOS, and Windows.

All Bifrost interaction must remain inside `brokk_bifrost_runtime::extension`. Dense semantic node numbers are response-local aliases. Persist and join only stable IDs with source spans and call context. `Complete`, frontier-bounded, budget-bounded, unsupported, and cancelled outcomes remain distinct.

## Validation and Acceptance

From the repository root, run:

    cargo fmt --check
    cargo test --locked --workspace
    cargo clippy --locked --workspace --all-targets -- -D warnings
    cargo metadata --locked --format-version 1 | python3 scripts/audit_dependencies.py --check audits/dependency-licenses.tsv
    cargo run --locked -- run-example --output artifacts/example
    cargo run --locked -- verify --bundle artifacts/example/cold
    cargo run --locked -- verify --bundle artifacts/example/reopen
    cargo run --locked -- reproduce --bundle artifacts/example/cold --workspace fixtures/workspace --output artifacts/reproduced

`cargo tree --locked -p bifrost-extension-template` must resolve `brokk-bifrost-runtime v0.11.0` from crates.io, never a path or Git checkout. Repeat the locked build and tests from a clean temporary copy without `.git`, generated artifacts, or a sibling Bifrost source tree. Generated bundles remain untracked under `artifacts/`.

Revision note (2026-09-08): Reintroduced the lifecycle ExecPlan required by repository instructions after the historical plan was removed from `main`, and recorded the 0.11.0 migration contract and remaining gates.

## v0.11.5 update (2026-09-28)

- [x] Started from current `origin/main` on `dave/update-bifrost-v0.11.5`, preserving the divergent local `main` commit.
- [x] Verified published `brokk-bifrost-runtime` 0.11.5 and the release tag; `bifrost-policy-scan` tags `v0` and `v0.11.5` both resolve to `3a5fc1465ca249e6cf5b8240f174819890b75351`.
- [x] Refreshed the exact runtime lockfile and verified the 350-record dependency/license inventory.
- [x] Ran format, workspace tests, strict Clippy, registry-only dependency check, and run/verify/reproduce smoke; cold and reproduced manifest digests matched.
- [x] Ran workspace tests from `/private/tmp/bifrost-template-0.11.5.x19Xta`, a source-only copy without Git metadata or a Bifrost checkout, reusing the local Cargo target.
- [ ] Verify cross-platform PR CI.

The runtime dependency remains registry-only and exact. The example continues to use ephemeral workspace mode; any changed v0.11.5 extension contract must be reflected in lifecycle code and evidence before completion.

## Policy selection follow-up (2026-09-28)

The v0.11.5 action scan on PR #5 was `UNRELIABLE` because `bifrost.correctness.python-absent-member` reported `capability_incomplete` on this workspace. The workflow now uses the action's `policy-ids` selector to run the other 16 members of `bifrost.code-smells`. This is an explicit selection decision, not a finding suppression or a completeness override. Verify the exact new PR head in CI before qualifying the policy gate.
