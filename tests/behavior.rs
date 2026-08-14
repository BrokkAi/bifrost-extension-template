use bifrost_extension_template::{RunOptions, run_lifecycle};
use brokk_bifrost_runtime::extension::*;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};

fn digest(bytes: &[u8]) -> StableDigest {
    StableDigest::parse(format!("{:x}", Sha256::digest(bytes))).unwrap()
}

fn workspace(root: &Path) -> ExtensionWorkspace {
    ExtensionWorkspace::open(ExtensionWorkspaceOptions::new(root)).unwrap()
}

fn observation_document(
    workspace: &ExtensionWorkspace,
    path: &str,
    source: &[u8],
    range: (u64, u64),
) -> ObservationDocument {
    ObservationDocument {
        schema_version: "1.0".into(),
        compatibility: ExtensionCompatibility::default(),
        expected_generation: workspace.generation().clone(),
        repository: ObservationRepository {
            kind: "fixture".into(),
            identity: "behavior".into(),
            revision: "1".repeat(40).into(),
            mount: "workspace".into(),
            dirty_overlay: None,
        },
        subject: identity("subject"),
        run: identity("run"),
        producer: ObservationProducer {
            format_name: "generic-range".into(),
            format_version: "1".into(),
            tool_name: "test".into(),
            tool_version: "1".into(),
            adapter_name: "test".into(),
            adapter_version: "1".into(),
            input_content_sha256: digest(b"input"),
        },
        configuration_hash: digest(b"configuration"),
        limits: ObservationMappingLimits {
            max_records: 4,
            max_source_bytes: 1 << 20,
            max_candidate_nodes: 128,
            max_mapped_nodes_per_record: 64,
            max_total_mapped_nodes: 128,
            max_diagnostics: 16,
            max_output_bytes: 1 << 20,
            max_materialized_files: 4,
            max_traversal_steps: 10_000,
        },
        include_synthetic: false,
        include_generated: false,
        records: vec![ObservationRecord {
            record_id: "record".into(),
            source: ObservationSourceIdentity {
                mount: "workspace".into(),
                span: SourceSpan {
                    path: NormalizedRelativePath::new(path).unwrap(),
                    start_utf8_byte: range.0,
                    end_utf8_byte: range.1,
                },
                content_sha256: digest(source),
                language: None,
                generated_provenance: None,
            },
            kind: ObservationRecordKind::Range,
            category: "event".into(),
            outcome: "seen".into(),
            attributes: BTreeMap::new(),
        }]
        .into_boxed_slice(),
    }
}

fn identity(id: &str) -> ObservationIdentity {
    ObservationIdentity {
        namespace: "test".into(),
        id: id.into(),
        category: "behavior".into(),
        outcome: "caller-owned".into(),
        attributes: BTreeMap::new(),
    }
}

#[test]
fn near_miss_snippet_is_rejected_before_mapping() {
    let temp = tempfile::tempdir().unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let input_path = temp.path().join("observations.json");
    let input = fs::read_to_string(root.join("fixtures/input/observations.json"))
        .unwrap()
        .replace("value = value * 2", "value = value * 3");
    fs::write(&input_path, input).unwrap();
    let error = run_lifecycle(&RunOptions {
        workspace: root.join("fixtures/workspace"),
        config: root.join("fixtures/input/config.json"),
        observations: input_path,
        output: temp.path().join("output"),
    })
    .unwrap_err();
    assert!(error.to_string().contains("source snippet not found"));
}

#[test]
fn unsupported_source_and_stale_content_are_typed_outcomes() {
    let temp = tempfile::tempdir().unwrap();
    let source = b"plain unsupported source\n";
    fs::write(temp.path().join("sample.txt"), source).unwrap();
    let workspace = workspace(temp.path());
    let document = observation_document(&workspace, "sample.txt", source, (0, 5));
    let result = workspace
        .map_observations(&document, &ExtensionCancellation::new())
        .unwrap();
    assert!(matches!(
        result.outcomes[0],
        ObservationMappingOutcome::Unsupported { .. }
    ));
    assert!(!result.complete);

    let mut stale = document;
    stale.records[0].source.content_sha256 = digest(b"different");
    let result = workspace
        .map_observations(&stale, &ExtensionCancellation::new())
        .unwrap();
    assert!(matches!(
        result.outcomes[0],
        ObservationMappingOutcome::Stale { .. }
    ));
    assert!(!result.complete);
}

#[test]
fn tiny_limits_and_cancellation_preserve_incompleteness() {
    let temp = tempfile::tempdir().unwrap();
    let source =
        b"function f(flag: boolean) { let value = 0; if (flag) value = 1; return value; }\n";
    fs::write(temp.path().join("sample.ts"), source).unwrap();
    let workspace = workspace(temp.path());
    let limits = ExtensionLimits::new(ExtensionLimitValues {
        semantic_nodes: 1,
        semantic_edges: 1,
        ..ExtensionLimitValues::default()
    })
    .unwrap();
    let request = SemanticRelationRequest::from_source(
        workspace.generation().clone(),
        SourceSpan {
            path: NormalizedRelativePath::new("sample.ts").unwrap(),
            start_utf8_byte: 10,
            end_utf8_byte: 10,
        },
        limits,
    )
    .unwrap();
    let truncated = workspace
        .semantic_relations(request.clone(), &ExtensionCancellation::new())
        .unwrap();
    assert!(matches!(
        truncated.completion,
        ExtensionCompletion::Truncated { .. }
    ));
    if let Some(snapshot) = truncated.value {
        assert!(!snapshot.authoritative_absence());
    }

    let cancellation = ExtensionCancellation::new();
    cancellation.cancel();
    let cancelled = workspace
        .semantic_relations(request, &cancellation)
        .unwrap();
    assert_eq!(cancelled.completion, ExtensionCompletion::Cancelled);
    assert!(cancelled.value.is_none());
}

#[test]
fn stale_generation_is_rejected_and_open_generation_is_immutable() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(
        temp.path().join("sample.ts"),
        "function f() { return 1; }\n",
    )
    .unwrap();
    let first = workspace(temp.path());
    let request = SemanticRelationRequest::from_source(
        first.generation().clone(),
        SourceSpan {
            path: NormalizedRelativePath::new("sample.ts").unwrap(),
            start_utf8_byte: 10,
            end_utf8_byte: 10,
        },
        ExtensionLimits::default(),
    )
    .unwrap();
    fs::write(
        temp.path().join("sample.ts"),
        "function f() { return 2; }\n",
    )
    .unwrap();
    let second = workspace(temp.path());
    assert_ne!(first.generation(), second.generation());
    assert!(matches!(
        second.semantic_relations(request, &ExtensionCancellation::new()),
        Err(ExtensionError::StaleGeneration { .. })
    ));
}

#[test]
fn partial_empty_snapshot_never_proves_absence() {
    let generation: WorkspaceGeneration =
        serde_json::from_str(&format!("\"{}\"", "a".repeat(64))).unwrap();
    let snapshot = SemanticRelationSnapshot::try_new(
        generation,
        digest(b"request"),
        SemanticRelationStatus::Partial,
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .unwrap();
    assert!(!snapshot.authoritative_absence());
}

#[test]
fn protocol_paths_reject_platform_specific_or_escaping_spellings() {
    for path in [
        "",
        "/absolute",
        "../escape",
        "src\\sample.ts",
        "src/./sample.ts",
    ] {
        assert!(NormalizedRelativePath::new(path).is_err(), "{path}");
    }
}
