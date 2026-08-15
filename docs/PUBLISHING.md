# Publication and citation guide

This source repository is public under Apache-2.0. Its Rust package intentionally has `publish = false`: do not upload a crate, create a package or template listing, or create a GitHub release without a separate review and explicit authorization.

## Source-publication evidence

The source-publication commit is required to satisfy all of the following:

1. Bifrost is pinned to the exact published Apache-2.0 `brokk-bifrost-runtime = "=0.10.1"`; there is no development Git or path dependency.
2. A clean isolated copy with no Bifrost source checkout passes the locked build, workspace tests, strict Clippy, and example lifecycle.
3. Linux, macOS, and Windows CI pass on the same commit.
4. Fresh runs compare deterministic artifacts and hashes, and direct Rust, JSON, and JSONL paths agree.
5. README, citation metadata, generic inputs, generated artifacts, and manifests have been inspected.
6. [`../audits/dependency-licenses.tsv`](../audits/dependency-licenses.tsv) is generated from the final locked graph and checked for drift in CI. All entries declare a license; all non-workspace entries use the crates.io registry source. This is a source audit, not a substitute for a future binary-distribution notices review.
7. Immediately before the visibility change, every remote branch, tag, and reachable commit is re-audited for obsolete licensing statements.
8. The tracked tree and repository refs contain no generated build/cache output, credentials, private fixtures, or copied Bifrost implementation code outside the documented extension surface.

These gates authorize public source visibility only. They do not authorize a crate upload, package or template listing, GitHub release, binary distribution, or support-status claim.

Repeat the relevant audits for any future distribution commit rather than inheriting this source-publication evidence.

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
