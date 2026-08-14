# Publication and citation guide

This repository is private and its Rust package has `publish = false`. Do not make it public, upload a crate, create a release, or describe it as supported until every gate below is evidenced on the intended release commit.

## Public-readiness gates

1. Keep Bifrost on an exact published Apache-2.0 dependency or executable. Development Git/path dependencies are not acceptable in a public release.
2. From a clean clone on a machine or isolated directory with no Bifrost source checkout, run `cargo build --locked`, `cargo test --locked --workspace`, strict Clippy, and the example lifecycle.
3. Require green Linux, macOS, and Windows private CI for the same commit.
4. Produce two fresh example runs and compare deterministic artifacts and hashes.
5. Inspect rendered README, citation metadata, generic input examples, generated artifacts, and manifests.
6. Generate and review the final locked dependency/license inventory and required notices.
7. Re-audit every branch, tag, and reachable commit for obsolete licensing statements immediately before visibility changes. The initial private `main` ancestry was replaced with a clean root on 2026-08-14 after PR #1 merged; do not assume that historical result covers later refs or commits.
8. Confirm the repository contains no generated build/cache output, credentials, private fixtures, Bifrost implementation code, or Joern materials.
9. Only after all gates pass, make a separate, explicitly authorized change to `publish`, visibility, package metadata, and release automation.

The current exact runtime pin and clean-clone checks can pass during private development. Keeping the gate in place ensures they are rerun on the actual public release commit rather than inherited from an earlier validation.

## Publishing result artifacts

Publish the verified bundle without altering its bytes. Report the manifest digest and include all components referenced by the manifest. Preserve raw incomplete outcomes and deviations; do not publish only the derived result.

State whether the bundle is conformance evidence, a development experiment, or a confirmatory result. A development bundle must not be relabeled after seeing its result. For confirmatory work, include the frozen protocol component required by the manifest purpose.

## Citation checklist

Include:

- Bifrost package version and source revision from `manifest.engine`;
- extension name, version, commit or package digest, and configuration digest;
- workspace repository/revision, generation, source inventory, roots, and exclusions;
- observation producer and input digest;
- relation and observation schema versions and component digests;
- request limits, diagnostics, work, completion, boundaries, and deviations;
- declared cold/rebuilt/reused cache state; and
- the immutable manifest digest and a durable artifact location.

Use `CITATION.cff` for the software citation and the manifest for the run-specific citation facts.
