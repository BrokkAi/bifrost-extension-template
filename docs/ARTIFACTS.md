# Artifact and evidence guide

Each run directory is a Bifrost extension bundle with canonical `manifest.json`, extension-owned inputs under `inputs/`, and evidence under `results/`.

## Components

- `inputs/config.json` is the versioned relation seed, scope, direction, budgets, and workspace identity.
- `inputs/observations.json` is the generic caller-owned observation source.
- `results/capabilities.json` binds the API and language capability report to the workspace generation.
- `results/observation-document.json` is the canonical Bifrost observation request after source identity is added.
- `results/observation-mapping.json` and `.jsonl` are equivalent terminal mapping outcomes.
- `results/relation-request.json` is the finite semantic request.
- `results/relation-response.json` retains completion, diagnostics, work, limits, generation, and provenance.
- `results/relation-snapshot.json` and `.jsonl` retain stable nodes, response-local aliases, typed edges, proof, completeness, evidence, boundaries, and digest.
- `results/observed-relations.json` is the extension-owned stable-ID join. It contains no dense snapshot identity.
- `results/transport-equivalence.json` records that direct Rust, JSON request/response, and JSONL round trips matched.

The manifest records byte hashes for every component and canonical semantic digests where the Bifrost schema supplies one. The derived result declares dependencies on the observation mapping and relation snapshot component hashes.

## Completion and absence

Observation mapping is complete only when every record ended exact or unmapped. Ambiguous, stale, unsupported, truncated, or cancelled records make it incomplete. A relation snapshot is complete only when its status is complete and the enclosing response completion is complete.

`authoritative_absence` is true only if both acquisitions are complete and Bifrost reports a complete snapshot with no relation edges. A partial empty result is preserved as partial evidence and never reworded as “no relationship exists.”

## Reopen declaration

`cold/manifest.json` declares a fully cold construction. `reopen/manifest.json` declares a same-process rebuild after one prior open. Source generation and semantic artifacts must be byte-identical between the two runs, but their manifest digests intentionally differ because cache-state evidence differs.

## Determinism

Canonical JSON uses lexicographically sorted object keys and a final newline. Bifrost codecs provide canonical ordering and content digests for observation, relation, JSONL, and manifest contracts. The template sorts all derived links by record, relation, and stable endpoints.

Volatile timestamps, hostnames, elapsed time, and memory readings are omitted. Two fresh runs against the same immutable workspace root and target/profile produce byte-identical bundles. A relocated checkout can have a different workspace generation and generation-bound semantic identities; `reproduce` reports that exact prerequisite mismatch. Cross-target manifest `engine.target` values intentionally differ and should be compared as declared provenance, not normalized away.
