use bifrost_extension_template::{RunOptions, reproduce_bundle, run_lifecycle, verify_bundle};
use brokk_bifrost_runtime::extension::{
    CacheStateKind, RunStatus, decode_canonical_run_manifest_json,
};
use std::{collections::BTreeMap, fs, path::Path};

fn fixture_options(output: &Path) -> RunOptions {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    RunOptions {
        workspace: root.join("fixtures/workspace"),
        config: root.join("fixtures/input/config.json"),
        observations: root.join("fixtures/input/observations.json"),
        output: output.to_path_buf(),
    }
}

#[test]
fn lifecycle_writes_verified_cold_and_reopen_bundles() {
    let temp = tempfile::tempdir().unwrap();
    let summary = run_lifecycle(&fixture_options(&temp.path().join("run"))).unwrap();
    let cold = temp.path().join("run/cold");
    let reopen = temp.path().join("run/reopen");
    assert_eq!(verify_bundle(&cold).unwrap(), summary.cold_manifest);
    assert_eq!(verify_bundle(&reopen).unwrap(), summary.reopen_manifest);
    assert_ne!(summary.cold_manifest, summary.reopen_manifest);

    let cold_manifest =
        decode_canonical_run_manifest_json(&fs::read(cold.join("manifest.json")).unwrap()).unwrap();
    let reopen_manifest =
        decode_canonical_run_manifest_json(&fs::read(reopen.join("manifest.json")).unwrap())
            .unwrap();
    assert_eq!(cold_manifest.cache.kind, CacheStateKind::FullyCold);
    assert_eq!(reopen_manifest.cache.kind, CacheStateKind::Rebuilt);
    assert!(reopen_manifest.cache.same_process);
    assert!(!reopen_manifest.cache.semantic_artifact_reused);
    assert_eq!(cold_manifest.status, RunStatus::Complete);

    let result = fs::read_to_string(cold.join("results/observed-relations.json")).unwrap();
    assert!(!result.contains("local_id"));
    assert!(!result.contains("dense"));
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(result["generation"], summary.generation.to_string());
    assert!(
        result["links"]
            .as_array()
            .is_some_and(|links| !links.is_empty())
    );
}

#[test]
fn independent_fresh_runs_are_byte_stable() {
    let temp = tempfile::tempdir().unwrap();
    run_lifecycle(&fixture_options(&temp.path().join("first"))).unwrap();
    run_lifecycle(&fixture_options(&temp.path().join("second"))).unwrap();
    assert_eq!(
        file_map(&temp.path().join("first")),
        file_map(&temp.path().join("second"))
    );
}

#[test]
fn lifecycle_refuses_to_overwrite_output() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("run");
    fs::create_dir(&output).unwrap();
    let error = run_lifecycle(&fixture_options(&output)).unwrap_err();
    assert!(error.to_string().contains("output already exists"));
}

#[test]
fn reproduction_recreates_exact_bundle_or_reports_generation_mismatch() {
    let temp = tempfile::tempdir().unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let original = temp.path().join("original");
    let summary = run_lifecycle(&fixture_options(&original)).unwrap();
    let reproduced = reproduce_bundle(
        &original.join("cold"),
        &root.join("fixtures/workspace"),
        &temp.path().join("reproduced"),
    )
    .unwrap();
    assert_eq!(reproduced.expected_manifest, summary.cold_manifest);
    assert_eq!(reproduced.reproduced_manifest, summary.cold_manifest);

    let relocated = temp.path().join("relocated");
    fs::create_dir(&relocated).unwrap();
    fs::create_dir(relocated.join("src")).unwrap();
    fs::copy(
        root.join("fixtures/workspace/src/sample.ts"),
        relocated.join("src/sample.ts"),
    )
    .unwrap();
    let error = reproduce_bundle(
        &original.join("cold"),
        &relocated,
        &temp.path().join("mismatch"),
    )
    .unwrap_err();
    assert!(error.to_string().contains("workspace.generation mismatch"));
}

fn file_map(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}

fn visit(root: &Path, directory: &Path, result: &mut BTreeMap<String, Vec<u8>>) {
    let mut entries = fs::read_dir(directory)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        if entry.file_type().unwrap().is_dir() {
            visit(root, &entry.path(), result);
        } else {
            let relative = entry
                .path()
                .strip_prefix(root)
                .unwrap()
                .components()
                .map(|part| part.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            result.insert(relative, fs::read(entry.path()).unwrap());
        }
    }
}
