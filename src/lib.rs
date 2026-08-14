//! Complete downstream example built only on Bifrost's documented extension API.

use brokk_bifrost_runtime::extension::{
    BundleVerificationLimits, CacheStateDeclaration, CacheStateKind, ComponentDependency,
    EXTENSION_API_VERSION, EngineRunIdentity, ExtensionCancellation, ExtensionCompatibility,
    ExtensionCompletion, ExtensionOutcome, ExtensionRequest, ExtensionResponse,
    ExtensionRunIdentity, ExtensionWorkspace, ExtensionWorkspaceDescription,
    ExtensionWorkspaceOptions, NormalizedRelativePath, ObservationDocument, ObservationIdentity,
    ObservationMappingLimits, ObservationMappingOutcome, ObservationMappingResult,
    ObservationProducer, ObservationRecord, ObservationRecordKind, ObservationRepository,
    ObservationScalar, ObservationSourceIdentity, RunComponentInput, RunManifestBuilder,
    RunPurpose, RunStatus, SemanticDirection, SemanticProof, SemanticRelationCompleteness,
    SemanticRelationKind, SemanticRelationLimits, SemanticRelationRequest, SemanticRelationScope,
    SemanticRelationSnapshot, SemanticRelationStatus, SemanticSeed, SourceSpan, StableDigest,
    WorkspaceGeneration, WorkspaceRunIdentity, decode_request_json,
    encode_observation_document_json, encode_observation_result_json, encode_relation_request_json,
    encode_relation_snapshot_json, encode_request_json, encode_response_json,
    encode_run_manifest_json, join_exact_observation, read_observation_mapping_jsonl,
    read_relation_snapshot_jsonl, verify_extension_bundle, write_observation_mapping_jsonl,
    write_relation_snapshot_jsonl,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, fs,
    io::Cursor,
    path::{Path, PathBuf},
};

const BIFROST_VERSION: &str = "0.9.5";
const BIFROST_PACKAGE_REVISION: &str = "a3ca30bd3fb994cc07db4abf47a2c796854882ca";
const TEMPLATE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Paths needed for one complete cold/reopen lifecycle.
#[derive(Debug, Clone)]
pub struct RunOptions {
    pub workspace: PathBuf,
    pub config: PathBuf,
    pub observations: PathBuf,
    pub output: PathBuf,
}

/// Stable identities printed after a successful lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSummary {
    pub generation: WorkspaceGeneration,
    pub cold_manifest: StableDigest,
    pub reopen_manifest: StableDigest,
}

/// Identity returned after reproducing the selected cold or reopen bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReproductionSummary {
    pub expected_manifest: StableDigest,
    pub reproduced_manifest: StableDigest,
}

/// User-facing error that never leaks Bifrost implementation types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateError(Box<str>);

impl TemplateError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into().into_boxed_str())
    }
}

impl fmt::Display for TemplateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for TemplateError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExtensionConfig {
    schema_version: Box<str>,
    workspace_repository: Box<str>,
    workspace_revision: Box<str>,
    seed_path: Box<str>,
    seed_needle: Box<str>,
    relations: Vec<SemanticRelationKind>,
    direction: SemanticDirection,
    limits: SemanticRelationLimits,
}

impl ExtensionConfig {
    fn validate(&self) -> Result<(), TemplateError> {
        if self.schema_version.as_ref() != "1.0" {
            return Err(TemplateError::new("unsupported configuration schema"));
        }
        if self.workspace_repository.is_empty()
            || self.workspace_revision.len() != 40
            || !self
                .workspace_revision
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
            || self.seed_needle.is_empty()
            || self.relations.is_empty()
        {
            return Err(TemplateError::new("invalid configuration identity or seed"));
        }
        NormalizedRelativePath::new(self.seed_path.as_ref()).map_err(TemplateError::new)?;
        self.limits
            .validate()
            .map_err(|error| TemplateError::new(error.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GenericObservationInput {
    schema_version: Box<str>,
    subject: GenericIdentity,
    run: GenericIdentity,
    producer: GenericProducer,
    records: Vec<GenericRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GenericIdentity {
    namespace: Box<str>,
    id: Box<str>,
    category: Box<str>,
    outcome: Box<str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GenericProducer {
    format_name: Box<str>,
    format_version: Box<str>,
    tool_name: Box<str>,
    tool_version: Box<str>,
    adapter_name: Box<str>,
    adapter_version: Box<str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GenericRecord {
    record_id: Box<str>,
    path: Box<str>,
    needle: Box<str>,
    category: Box<str>,
    outcome: Box<str>,
    #[serde(default)]
    attributes: BTreeMap<Box<str>, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AnalysisResult {
    schema_version: &'static str,
    generation: WorkspaceGeneration,
    observation_input_digest: StableDigest,
    observation_result_digest: StableDigest,
    relation_request_digest: StableDigest,
    relation_snapshot_digest: StableDigest,
    acquisition_complete: bool,
    authoritative_absence: bool,
    observation_outcomes: Vec<ObservationOutcomeSummary>,
    links: Vec<ObservedRelationLink>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
struct ObservationOutcomeSummary {
    record_id: Box<str>,
    status: Box<str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ObservedRelationLink {
    record_id: Box<str>,
    relation: SemanticRelationKind,
    source_stable_id: StableDigest,
    target_stable_id: StableDigest,
    proof: SemanticProof,
    completeness: SemanticRelationCompleteness,
    source_mappings: Vec<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct EquivalenceReport {
    direct_equals_json: bool,
    direct_equals_jsonl: bool,
    dense_ids_persisted_by_analysis: bool,
}

struct RunEvidence {
    description: ExtensionWorkspaceDescription,
    config_bytes: Vec<u8>,
    input_bytes: Vec<u8>,
    capability_bytes: Vec<u8>,
    observation_document_bytes: Vec<u8>,
    observation_result_bytes: Vec<u8>,
    observation_jsonl_bytes: Vec<u8>,
    relation_request_bytes: Vec<u8>,
    relation_response_bytes: Vec<u8>,
    relation_snapshot_bytes: Vec<u8>,
    relation_jsonl_bytes: Vec<u8>,
    analysis_bytes: Vec<u8>,
    equivalence_bytes: Vec<u8>,
    observation_digest: StableDigest,
    relation_digest: StableDigest,
    relation_status: RunStatus,
}

struct ObservationEvidence {
    result: ObservationMappingResult,
    document_bytes: Vec<u8>,
    result_bytes: Vec<u8>,
    jsonl_bytes: Vec<u8>,
}

struct RelationEvidence {
    outcome: ExtensionOutcome<SemanticRelationSnapshot>,
    request_bytes: Vec<u8>,
    response_bytes: Vec<u8>,
    snapshot_bytes: Vec<u8>,
    jsonl_bytes: Vec<u8>,
}

struct ComponentBytes {
    role: &'static str,
    path: &'static str,
    media_type: &'static str,
    schema: Option<&'static str>,
    bytes: Vec<u8>,
    canonical_digest: Option<StableDigest>,
    status: RunStatus,
    dependencies: Vec<(&'static str, StableDigest)>,
}

impl ComponentBytes {
    fn json(role: &'static str, path: &'static str, bytes: Vec<u8>) -> Self {
        Self {
            role,
            path,
            media_type: "application/json",
            schema: None,
            bytes,
            canonical_digest: None,
            status: RunStatus::Complete,
            dependencies: Vec::new(),
        }
    }

    fn jsonl(mut self) -> Self {
        self.media_type = "application/x-ndjson";
        self
    }

    fn schema(mut self, schema: &'static str) -> Self {
        self.schema = Some(schema);
        self
    }

    fn digest(mut self, digest: StableDigest) -> Self {
        self.canonical_digest = Some(digest);
        self
    }

    fn status(mut self, status: RunStatus) -> Self {
        self.status = status;
        self
    }

    fn depends(mut self, role: &'static str, digest: StableDigest) -> Self {
        self.dependencies.push((role, digest));
        self
    }
}

/// Execute a cold run, reopen the immutable workspace, and write two verified bundles.
///
/// # Errors
///
/// Returns an error when inputs are invalid, Bifrost cannot acquire evidence, direct and
/// serialized paths differ, the output already exists, or bundle verification fails.
pub fn run_lifecycle(options: &RunOptions) -> Result<RunSummary, TemplateError> {
    if options.output.exists() {
        return Err(TemplateError::new(format!(
            "output already exists: {}",
            options.output.display()
        )));
    }
    let config_bytes = read(&options.config)?;
    let input_bytes = read(&options.observations)?;
    let config: ExtensionConfig = strict_json(&config_bytes, "configuration")?;
    config.validate()?;
    let input: GenericObservationInput = strict_json(&input_bytes, "observation input")?;
    validate_generic_input(&input)?;

    let cold_workspace = open_workspace(&options.workspace)?;
    let cold = collect_evidence(
        &cold_workspace,
        &options.workspace,
        &config,
        &input,
        &config_bytes,
        &input_bytes,
    )?;

    let reopen_workspace = open_workspace(&options.workspace)?;
    if cold_workspace.generation() != reopen_workspace.generation() {
        return Err(TemplateError::new(
            "unchanged workspace produced a different generation on reopen",
        ));
    }
    let reopen = collect_evidence(
        &reopen_workspace,
        &options.workspace,
        &config,
        &input,
        &config_bytes,
        &input_bytes,
    )?;
    if cold.analysis_bytes != reopen.analysis_bytes
        || cold.relation_snapshot_bytes != reopen.relation_snapshot_bytes
        || cold.observation_result_bytes != reopen.observation_result_bytes
    {
        return Err(TemplateError::new(
            "reopen changed deterministic semantic artifacts",
        ));
    }

    fs::create_dir_all(&options.output)
        .map_err(|error| TemplateError::new(format!("create output: {error}")))?;
    let cold_manifest = write_bundle(
        &options.output.join("cold"),
        &cold,
        &options.workspace,
        &config,
        CacheStateDeclaration {
            kind: CacheStateKind::FullyCold,
            same_process: false,
            persisted_source: false,
            semantic_artifact_reused: false,
            warmup_count: 0,
        },
    )?;
    let reopen_manifest = write_bundle(
        &options.output.join("reopen"),
        &reopen,
        &options.workspace,
        &config,
        CacheStateDeclaration {
            kind: CacheStateKind::Rebuilt,
            same_process: true,
            persisted_source: false,
            semantic_artifact_reused: false,
            warmup_count: 1,
        },
    )?;

    Ok(RunSummary {
        generation: cold_workspace.generation().clone(),
        cold_manifest,
        reopen_manifest,
    })
}

/// Verify a bundle with Bifrost's bounded #2105 validator and return its digest.
///
/// # Errors
///
/// Returns an error when the manifest is malformed, noncanonical, inconsistent, or when a
/// component is absent, unsafe, oversized, or fails its declared content hash.
pub fn verify_bundle(root: &Path) -> Result<StableDigest, TemplateError> {
    let verified =
        verify_extension_bundle(root, BundleVerificationLimits::default()).map_err(|errors| {
            TemplateError::new(
                errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; "),
            )
        })?;
    Ok(verified.manifest().manifest_digest.clone())
}

/// Recreate a verified bundle from its embedded inputs and an exact workspace prerequisite.
///
/// # Errors
///
/// Returns an error when bundle verification fails, the workspace generation or source inventory
/// differs, required input roles are absent, execution fails, or the recreated digest differs.
pub fn reproduce_bundle(
    bundle: &Path,
    workspace_root: &Path,
    output: &Path,
) -> Result<ReproductionSummary, TemplateError> {
    let verified =
        verify_extension_bundle(bundle, BundleVerificationLimits::default()).map_err(|errors| {
            TemplateError::new(
                errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; "),
            )
        })?;
    let manifest = verified.manifest();
    let workspace = open_workspace(workspace_root)?;
    if workspace.generation() != &manifest.workspace.generation {
        return Err(TemplateError::new(format!(
            "workspace.generation mismatch: expected {}, observed {}",
            manifest.workspace.generation,
            workspace.generation()
        )));
    }
    let inventory = source_inventory_digest(workspace_root)?;
    if inventory != manifest.workspace.source_inventory_digest {
        return Err(TemplateError::new(format!(
            "workspace.source_inventory_digest mismatch: expected {}, observed {inventory}",
            manifest.workspace.source_inventory_digest
        )));
    }
    let config = component_path(bundle, manifest, "configuration")?;
    let observations = component_path(bundle, manifest, "generic-observations")?;
    let reproduced = run_lifecycle(&RunOptions {
        workspace: workspace_root.to_path_buf(),
        config,
        observations,
        output: output.to_path_buf(),
    })?;
    let reproduced_manifest = match manifest.cache.kind {
        CacheStateKind::FullyCold => reproduced.cold_manifest,
        CacheStateKind::Rebuilt => reproduced.reopen_manifest,
        kind => {
            return Err(TemplateError::new(format!(
                "cache.kind prerequisite unsupported by this template: {kind:?}"
            )));
        }
    };
    if reproduced_manifest != manifest.manifest_digest {
        return Err(TemplateError::new(format!(
            "manifest_digest mismatch: expected {}, observed {reproduced_manifest}",
            manifest.manifest_digest
        )));
    }
    Ok(ReproductionSummary {
        expected_manifest: manifest.manifest_digest.clone(),
        reproduced_manifest,
    })
}

fn component_path(
    root: &Path,
    manifest: &brokk_bifrost_runtime::extension::ExtensionRunManifest,
    role: &str,
) -> Result<PathBuf, TemplateError> {
    manifest
        .components
        .iter()
        .find(|component| component.role.as_ref() == role)
        .map(|component| root.join(component.path.as_str()))
        .ok_or_else(|| TemplateError::new(format!("manifest component role absent: {role}")))
}

fn open_workspace(root: &Path) -> Result<ExtensionWorkspace, TemplateError> {
    ExtensionWorkspace::open(ExtensionWorkspaceOptions::new(root))
        .map_err(|error| TemplateError::new(format!("open workspace {}: {error}", root.display())))
}

fn collect_evidence(
    workspace: &ExtensionWorkspace,
    workspace_root: &Path,
    config: &ExtensionConfig,
    input: &GenericObservationInput,
    config_bytes: &[u8],
    input_bytes: &[u8],
) -> Result<RunEvidence, TemplateError> {
    let description = workspace.describe();
    let capability_bytes = canonical_json(&description)?;
    let document = observation_document(workspace, workspace_root, config, input, input_bytes)?;
    let observations = execute_observations(workspace, &document)?;
    let relation_request = relation_request(workspace, workspace_root, config)?;
    let relations = execute_relations(workspace, relation_request)?;
    let snapshot = relations.outcome.value.as_ref().ok_or_else(|| {
        TemplateError::new(format!(
            "no semantic snapshot: {:?}",
            relations.outcome.completion
        ))
    })?;
    let analysis = analyze(&observations.result, &relations.outcome, snapshot)?;
    let analysis_bytes = canonical_json(&analysis)?;
    let equivalence_bytes = canonical_json(&EquivalenceReport {
        direct_equals_json: true,
        direct_equals_jsonl: true,
        dense_ids_persisted_by_analysis: false,
    })?;
    let relation_status = completion_status(&relations.outcome.completion, snapshot.status);

    Ok(RunEvidence {
        description,
        config_bytes: config_bytes.to_vec(),
        input_bytes: input_bytes.to_vec(),
        capability_bytes,
        observation_document_bytes: observations.document_bytes,
        observation_result_bytes: observations.result_bytes,
        observation_jsonl_bytes: observations.jsonl_bytes,
        relation_request_bytes: relations.request_bytes,
        relation_response_bytes: relations.response_bytes,
        relation_snapshot_bytes: relations.snapshot_bytes,
        relation_jsonl_bytes: relations.jsonl_bytes,
        analysis_bytes,
        equivalence_bytes,
        observation_digest: observations.result.result_digest,
        relation_digest: snapshot.snapshot_digest.clone(),
        relation_status,
    })
}

fn execute_observations(
    workspace: &ExtensionWorkspace,
    document: &ObservationDocument,
) -> Result<ObservationEvidence, TemplateError> {
    let document_bytes = encode_observation_document_json(document)
        .map_err(|error| TemplateError::new(error.to_string()))?;
    let direct = workspace
        .map_observations(document, &ExtensionCancellation::new())
        .map_err(|error| TemplateError::new(error.to_string()))?;
    let encoded_request =
        encode_request_json(&ExtensionRequest::Observations(Box::new(document.clone())))
            .map_err(|error| TemplateError::new(error.to_string()))?;
    let request = decode_request_json(&encoded_request)
        .map_err(|error| TemplateError::new(error.to_string()))?;
    let ExtensionResponse::Observations(serialized) = workspace
        .execute(request, &ExtensionCancellation::new())
        .map_err(|error| TemplateError::new(error.to_string()))?
    else {
        return Err(TemplateError::new(
            "observation request changed response kind",
        ));
    };
    if direct != serialized {
        return Err(TemplateError::new(
            "direct and serialized observation results differ",
        ));
    }
    let result_bytes = encode_observation_result_json(&direct)
        .map_err(|error| TemplateError::new(error.to_string()))?;
    let mut jsonl_bytes = Vec::new();
    write_observation_mapping_jsonl(&direct, &mut jsonl_bytes)
        .map_err(|error| TemplateError::new(error.to_string()))?;
    let jsonl = read_observation_mapping_jsonl(Cursor::new(&jsonl_bytes))
        .map_err(|error| TemplateError::new(error.to_string()))?;
    if jsonl != direct {
        return Err(TemplateError::new("observation JSONL round trip differs"));
    }
    Ok(ObservationEvidence {
        result: direct,
        document_bytes,
        result_bytes,
        jsonl_bytes,
    })
}

fn execute_relations(
    workspace: &ExtensionWorkspace,
    request: SemanticRelationRequest,
) -> Result<RelationEvidence, TemplateError> {
    let request_bytes = encode_relation_request_json(&request)
        .map_err(|error| TemplateError::new(error.to_string()))?;
    let direct = workspace
        .semantic_relations(request.clone(), &ExtensionCancellation::new())
        .map_err(|error| TemplateError::new(error.to_string()))?;
    let encoded_request = encode_request_json(&ExtensionRequest::SemanticRelations(request))
        .map_err(|error| TemplateError::new(error.to_string()))?;
    let request = decode_request_json(&encoded_request)
        .map_err(|error| TemplateError::new(error.to_string()))?;
    let serialized_response = workspace
        .execute(request, &ExtensionCancellation::new())
        .map_err(|error| TemplateError::new(error.to_string()))?;
    let ExtensionResponse::SemanticRelations(serialized) = &serialized_response else {
        return Err(TemplateError::new("semantic request changed response kind"));
    };
    if serialized != &direct {
        return Err(TemplateError::new(
            "direct and serialized semantic results differ",
        ));
    }
    let response_bytes = canonicalize_json_bytes(
        &encode_response_json(&serialized_response)
            .map_err(|error| TemplateError::new(error.to_string()))?,
    )?;
    let snapshot = direct.value.as_ref().ok_or_else(|| {
        TemplateError::new(format!("no semantic snapshot: {:?}", direct.completion))
    })?;
    let snapshot_bytes = encode_relation_snapshot_json(snapshot)
        .map_err(|error| TemplateError::new(error.to_string()))?;
    let mut jsonl_bytes = Vec::new();
    write_relation_snapshot_jsonl(snapshot, &mut jsonl_bytes)
        .map_err(|error| TemplateError::new(error.to_string()))?;
    let jsonl = read_relation_snapshot_jsonl(Cursor::new(&jsonl_bytes))
        .map_err(|error| TemplateError::new(error.to_string()))?;
    if &jsonl != snapshot {
        return Err(TemplateError::new("semantic JSONL round trip differs"));
    }
    Ok(RelationEvidence {
        outcome: direct,
        request_bytes,
        response_bytes,
        snapshot_bytes,
        jsonl_bytes,
    })
}

fn observation_document(
    workspace: &ExtensionWorkspace,
    workspace_root: &Path,
    config: &ExtensionConfig,
    input: &GenericObservationInput,
    raw_input: &[u8],
) -> Result<ObservationDocument, TemplateError> {
    let records = input
        .records
        .iter()
        .map(|record| observation_record(workspace_root, record))
        .collect::<Result<Vec<_>, _>>()?;
    let document = ObservationDocument {
        schema_version: "1.0".into(),
        compatibility: ExtensionCompatibility::default(),
        expected_generation: workspace.generation().clone(),
        repository: ObservationRepository {
            kind: "fixture".into(),
            identity: config.workspace_repository.clone(),
            revision: config.workspace_revision.clone(),
            mount: "workspace".into(),
            dirty_overlay: None,
        },
        subject: identity(&input.subject),
        run: identity(&input.run),
        producer: ObservationProducer {
            format_name: input.producer.format_name.clone(),
            format_version: input.producer.format_version.clone(),
            tool_name: input.producer.tool_name.clone(),
            tool_version: input.producer.tool_version.clone(),
            adapter_name: input.producer.adapter_name.clone(),
            adapter_version: input.producer.adapter_version.clone(),
            input_content_sha256: sha256(raw_input),
        },
        configuration_hash: sha256(&canonical_json(config)?),
        limits: ObservationMappingLimits {
            max_records: 128,
            max_source_bytes: 4 << 20,
            max_candidate_nodes: 1024,
            max_mapped_nodes_per_record: 256,
            max_total_mapped_nodes: 2048,
            max_diagnostics: 128,
            max_output_bytes: 8 << 20,
            max_materialized_files: 64,
            max_traversal_steps: 1_000_000,
        },
        include_synthetic: false,
        include_generated: false,
        records: records.into_boxed_slice(),
    };
    document
        .validate()
        .map_err(|error| TemplateError::new(error.to_string()))?;
    Ok(document)
}

fn observation_record(
    workspace_root: &Path,
    record: &GenericRecord,
) -> Result<ObservationRecord, TemplateError> {
    let normalized =
        NormalizedRelativePath::new(record.path.as_ref()).map_err(TemplateError::new)?;
    let source = read(&workspace_root.join(normalized.as_str()))?;
    let source_text = std::str::from_utf8(&source)
        .map_err(|error| TemplateError::new(format!("source is not UTF-8: {error}")))?;
    let (start, end) = unique_span(source_text, &record.needle)?;
    Ok(ObservationRecord {
        record_id: record.record_id.clone(),
        source: ObservationSourceIdentity {
            mount: "workspace".into(),
            span: SourceSpan {
                path: normalized,
                start_utf8_byte: start,
                end_utf8_byte: end,
            },
            content_sha256: sha256(&source),
            language: None,
            generated_provenance: None,
        },
        kind: ObservationRecordKind::Range,
        category: record.category.clone(),
        outcome: record.outcome.clone(),
        attributes: scalar_attributes(&record.attributes)?,
    })
}

fn relation_request(
    workspace: &ExtensionWorkspace,
    workspace_root: &Path,
    config: &ExtensionConfig,
) -> Result<SemanticRelationRequest, TemplateError> {
    let path =
        NormalizedRelativePath::new(config.seed_path.as_ref()).map_err(TemplateError::new)?;
    let source = read(&workspace_root.join(path.as_str()))?;
    let source_text = std::str::from_utf8(&source)
        .map_err(|error| TemplateError::new(format!("source is not UTF-8: {error}")))?;
    let (start, end) = unique_span(source_text, &config.seed_needle)?;
    SemanticRelationRequest::new(
        workspace.generation().clone(),
        vec![SemanticSeed::Source {
            span: SourceSpan {
                path,
                start_utf8_byte: start,
                end_utf8_byte: end,
            },
        }],
        SemanticRelationScope::Procedure,
        config.relations.clone(),
        config.direction,
        config.limits,
    )
    .map_err(|error| TemplateError::new(error.to_string()))
}

fn analyze(
    observations: &ObservationMappingResult,
    relation_outcome: &ExtensionOutcome<SemanticRelationSnapshot>,
    snapshot: &SemanticRelationSnapshot,
) -> Result<AnalysisResult, TemplateError> {
    let local_to_stable = snapshot
        .nodes
        .iter()
        .map(|node| (node.local_id, node.stable_id.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut links = Vec::new();
    for outcome in &observations.outcomes {
        if !matches!(outcome, ObservationMappingOutcome::Exact { .. }) {
            continue;
        }
        let joined = join_exact_observation(outcome, snapshot, None)
            .map_err(|error| TemplateError::new(error.to_string()))?;
        let matched = joined
            .matches
            .iter()
            .map(|(_, stable)| stable)
            .collect::<BTreeSet<_>>();
        for edge in &snapshot.edges {
            let source = local_to_stable
                .get(&edge.source)
                .ok_or_else(|| TemplateError::new("dangling relation source"))?;
            let target = local_to_stable
                .get(&edge.target)
                .ok_or_else(|| TemplateError::new("dangling relation target"))?;
            if !matched.contains(source) && !matched.contains(target) {
                continue;
            }
            let mut mappings = edge
                .evidence
                .iter()
                .flat_map(|evidence| evidence.mappings.iter().cloned())
                .collect::<Vec<_>>();
            mappings.sort_by(|left, right| {
                (left.path.as_str(), left.start_utf8_byte, left.end_utf8_byte).cmp(&(
                    right.path.as_str(),
                    right.start_utf8_byte,
                    right.end_utf8_byte,
                ))
            });
            mappings.dedup();
            links.push(ObservedRelationLink {
                record_id: joined.record_id.clone(),
                relation: edge.kind,
                source_stable_id: source.clone(),
                target_stable_id: target.clone(),
                proof: edge.proof.clone(),
                completeness: edge.completeness.clone(),
                source_mappings: mappings,
            });
        }
    }
    links.sort_by(|left, right| {
        (
            &left.record_id,
            left.relation,
            &left.source_stable_id,
            &left.target_stable_id,
            &left.proof,
            &left.completeness,
        )
            .cmp(&(
                &right.record_id,
                right.relation,
                &right.source_stable_id,
                &right.target_stable_id,
                &right.proof,
                &right.completeness,
            ))
    });
    links.dedup();
    let mut outcomes = observations
        .outcomes
        .iter()
        .map(|outcome| ObservationOutcomeSummary {
            record_id: outcome.record_id().into(),
            status: observation_status(outcome).into(),
        })
        .collect::<Vec<_>>();
    outcomes.sort();
    let acquisition_complete = observations.complete
        && relation_outcome.completion == ExtensionCompletion::Complete
        && snapshot.status == SemanticRelationStatus::Complete;
    Ok(AnalysisResult {
        schema_version: "1.0",
        generation: snapshot.generation.clone(),
        observation_input_digest: observations.input_digest.clone(),
        observation_result_digest: observations.result_digest.clone(),
        relation_request_digest: snapshot.request_digest.clone(),
        relation_snapshot_digest: snapshot.snapshot_digest.clone(),
        acquisition_complete,
        authoritative_absence: acquisition_complete && snapshot.authoritative_absence(),
        observation_outcomes: outcomes,
        links,
    })
}

fn write_bundle(
    root: &Path,
    evidence: &RunEvidence,
    workspace_root: &Path,
    config: &ExtensionConfig,
    cache: CacheStateDeclaration,
) -> Result<StableDigest, TemplateError> {
    fs::create_dir_all(root)
        .map_err(|error| TemplateError::new(format!("create bundle: {error}")))?;
    let components = bundle_components(evidence);
    write_component_files(root, &components)?;
    let manifest = build_manifest(evidence, workspace_root, config, cache, &components)?;
    let manifest_bytes = encode_run_manifest_json(&manifest)
        .map_err(|error| TemplateError::new(error.to_string()))?;
    fs::write(root.join("manifest.json"), manifest_bytes)
        .map_err(|error| TemplateError::new(format!("write manifest: {error}")))?;
    let verified_digest = verify_bundle(root)?;
    if verified_digest != manifest.manifest_digest {
        return Err(TemplateError::new("verified manifest digest changed"));
    }
    Ok(verified_digest)
}

fn bundle_components(evidence: &RunEvidence) -> Vec<ComponentBytes> {
    let mut components = base_components(evidence);
    components.extend(observation_components(evidence));
    components.extend(relation_components(evidence));
    components
}

fn base_components(evidence: &RunEvidence) -> Vec<ComponentBytes> {
    vec![
        ComponentBytes::json(
            "configuration",
            "inputs/config.json",
            evidence.config_bytes.clone(),
        )
        .schema("1.0"),
        ComponentBytes::json(
            "generic-observations",
            "inputs/observations.json",
            evidence.input_bytes.clone(),
        )
        .schema("1.0"),
        ComponentBytes::json(
            "capability-report",
            "results/capabilities.json",
            evidence.capability_bytes.clone(),
        )
        .schema("bifrost-extension-api-1.0"),
        ComponentBytes::json(
            "transport-equivalence",
            "results/transport-equivalence.json",
            evidence.equivalence_bytes.clone(),
        )
        .schema("bifrost-extension-template-equivalence-1.0"),
    ]
}

fn observation_components(evidence: &RunEvidence) -> Vec<ComponentBytes> {
    vec![
        ComponentBytes::json(
            "observation-document",
            "results/observation-document.json",
            evidence.observation_document_bytes.clone(),
        )
        .schema("bifrost-observation-1.0"),
        ComponentBytes::json(
            "observation-mapping",
            "results/observation-mapping.json",
            evidence.observation_result_bytes.clone(),
        )
        .schema("bifrost-observation-1.0")
        .digest(evidence.observation_digest.clone()),
        ComponentBytes::json(
            "observation-mapping-jsonl",
            "results/observation-mapping.jsonl",
            evidence.observation_jsonl_bytes.clone(),
        )
        .jsonl()
        .schema("bifrost-observation-1.0")
        .digest(evidence.observation_digest.clone()),
    ]
}

fn relation_components(evidence: &RunEvidence) -> Vec<ComponentBytes> {
    let observation_hash = sha256(&evidence.observation_result_bytes);
    let relation_hash = sha256(&evidence.relation_snapshot_bytes);
    vec![
        ComponentBytes::json(
            "relation-request",
            "results/relation-request.json",
            evidence.relation_request_bytes.clone(),
        )
        .schema("bifrost-relation-1.0"),
        ComponentBytes::json(
            "relation-response",
            "results/relation-response.json",
            evidence.relation_response_bytes.clone(),
        )
        .schema("bifrost-extension-api-1.0")
        .status(evidence.relation_status),
        ComponentBytes::json(
            "relation-snapshot",
            "results/relation-snapshot.json",
            evidence.relation_snapshot_bytes.clone(),
        )
        .schema("bifrost-relation-1.0")
        .digest(evidence.relation_digest.clone())
        .status(evidence.relation_status),
        ComponentBytes::json(
            "relation-snapshot-jsonl",
            "results/relation-snapshot.jsonl",
            evidence.relation_jsonl_bytes.clone(),
        )
        .jsonl()
        .schema("bifrost-relation-1.0")
        .digest(evidence.relation_digest.clone())
        .status(evidence.relation_status),
        ComponentBytes::json(
            "observed-relations",
            "results/observed-relations.json",
            evidence.analysis_bytes.clone(),
        )
        .schema("bifrost-extension-template-result-1.0")
        .status(evidence.relation_status)
        .depends("observation-mapping", observation_hash)
        .depends("relation-snapshot", relation_hash),
    ]
}

fn write_component_files(root: &Path, components: &[ComponentBytes]) -> Result<(), TemplateError> {
    for item in components {
        let path = root.join(item.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                TemplateError::new(format!("create artifact directory: {error}"))
            })?;
        }
        fs::write(&path, &item.bytes)
            .map_err(|error| TemplateError::new(format!("write {}: {error}", path.display())))?;
    }
    Ok(())
}

fn build_manifest(
    evidence: &RunEvidence,
    workspace_root: &Path,
    config: &ExtensionConfig,
    cache: CacheStateDeclaration,
    components: &[ComponentBytes],
) -> Result<brokk_bifrost_runtime::extension::ExtensionRunManifest, TemplateError> {
    let capability_digest = sha256(
        components
            .iter()
            .find(|item| item.role == "capability-report")
            .map_or(&[][..], |item| item.bytes.as_slice()),
    );
    let config_digest = sha256(
        components
            .iter()
            .find(|item| item.role == "configuration")
            .map_or(&[][..], |item| item.bytes.as_slice()),
    );
    let mut adapters = evidence
        .description
        .capabilities
        .languages
        .iter()
        .map(|language| format!("{}:extension-v1", language.language).into_boxed_str())
        .collect::<Vec<_>>();
    adapters.sort();
    let engine = EngineRunIdentity {
        package_version: BIFROST_VERSION.into(),
        commit: BIFROST_PACKAGE_REVISION.into(),
        dirty_tree: None,
        profile: if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
        .into(),
        target: format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS).into(),
        features: Box::new([]),
        extension_api: EXTENSION_API_VERSION,
        semantic_ir_versions: vec!["1.0".into()].into_boxed_slice(),
        adapter_identities: adapters.into_boxed_slice(),
        capability_report_digest: capability_digest,
    };
    let workspace_identity = WorkspaceRunIdentity {
        repository: config.workspace_repository.clone(),
        commit: config.workspace_revision.clone(),
        dirty_tree: None,
        generation: evidence.description.generation.clone(),
        source_inventory_digest: source_inventory_digest(workspace_root)?,
        roots: vec![".".into()].into_boxed_slice(),
        exclusions: Box::new([]),
        dependency_fingerprints: Box::new([]),
    };
    let extension = ExtensionRunIdentity {
        name: "bifrost-extension-template".into(),
        version: TEMPLATE_VERSION.into(),
        commit: None,
        package_digest: Some(sha256(include_bytes!("../Cargo.lock"))),
        configuration_digest: config_digest,
    };
    let mut builder = RunManifestBuilder::from_workspace(
        &evidence.description,
        engine,
        workspace_identity,
        extension,
        RunPurpose::DevelopmentExperiment {
            objective: "demonstrate bounded generic observation and relation composition".into(),
        },
        cache,
    )
    .map_err(|error| TemplateError::new(error.to_string()))?;
    for item in components {
        builder = builder
            .add_component(RunComponentInput {
                role: item.role.into(),
                path: NormalizedRelativePath::new(item.path).map_err(TemplateError::new)?,
                media_type: item.media_type.into(),
                schema: item.schema.map(Into::into),
                bytes: &item.bytes,
                canonical_digest: item.canonical_digest.clone(),
                status: item.status,
                dependencies: item
                    .dependencies
                    .iter()
                    .map(|(role, digest)| ComponentDependency {
                        role: (*role).into(),
                        digest: digest.clone(),
                    })
                    .collect(),
            })
            .map_err(|error| TemplateError::new(error.to_string()))?;
    }
    let aggregate = if evidence.relation_status == RunStatus::Complete {
        RunStatus::Complete
    } else {
        RunStatus::Incomplete
    };
    builder
        .build(aggregate)
        .map_err(|error| TemplateError::new(error.to_string()))
}

fn validate_generic_input(input: &GenericObservationInput) -> Result<(), TemplateError> {
    if input.schema_version.as_ref() != "1.0" || input.records.is_empty() {
        return Err(TemplateError::new(
            "invalid generic observation schema or records",
        ));
    }
    let mut ids = BTreeSet::new();
    for record in &input.records {
        if record.record_id.is_empty()
            || !ids.insert(record.record_id.as_ref())
            || record.needle.is_empty()
        {
            return Err(TemplateError::new(
                "observation record IDs must be unique and needles nonempty",
            ));
        }
        NormalizedRelativePath::new(record.path.as_ref()).map_err(TemplateError::new)?;
    }
    Ok(())
}

fn identity(value: &GenericIdentity) -> ObservationIdentity {
    ObservationIdentity {
        namespace: value.namespace.clone(),
        id: value.id.clone(),
        category: value.category.clone(),
        outcome: value.outcome.clone(),
        attributes: BTreeMap::new(),
    }
}

fn scalar_attributes(
    values: &BTreeMap<Box<str>, Value>,
) -> Result<BTreeMap<Box<str>, ObservationScalar>, TemplateError> {
    values
        .iter()
        .map(|(key, value)| {
            let scalar = match value {
                Value::String(value) => ObservationScalar::String(value.clone().into_boxed_str()),
                Value::Bool(value) => ObservationScalar::Boolean(*value),
                Value::Number(value) => value
                    .as_i64()
                    .map(ObservationScalar::Signed)
                    .or_else(|| value.as_u64().map(ObservationScalar::Unsigned))
                    .ok_or_else(|| TemplateError::new("observation numbers must be integers"))?,
                _ => return Err(TemplateError::new("observation attributes must be scalar")),
            };
            Ok((key.clone(), scalar))
        })
        .collect()
}

fn unique_span(source: &str, needle: &str) -> Result<(u64, u64), TemplateError> {
    let mut matches = source.match_indices(needle);
    let Some((start, _)) = matches.next() else {
        return Err(TemplateError::new(format!(
            "source snippet not found: {needle}"
        )));
    };
    if matches.next().is_some() {
        return Err(TemplateError::new(format!(
            "source snippet is ambiguous: {needle}"
        )));
    }
    Ok((start as u64, (start + needle.len()) as u64))
}

fn observation_status(outcome: &ObservationMappingOutcome) -> &'static str {
    match outcome {
        ObservationMappingOutcome::Exact { .. } => "exact",
        ObservationMappingOutcome::Ambiguous { .. } => "ambiguous",
        ObservationMappingOutcome::Unmapped { .. } => "unmapped",
        ObservationMappingOutcome::Stale { .. } => "stale",
        ObservationMappingOutcome::Unsupported { .. } => "unsupported",
        ObservationMappingOutcome::Truncated { .. } => "truncated",
    }
}

fn completion_status(
    completion: &ExtensionCompletion,
    snapshot: SemanticRelationStatus,
) -> RunStatus {
    match completion {
        ExtensionCompletion::Complete if snapshot == SemanticRelationStatus::Complete => {
            RunStatus::Complete
        }
        ExtensionCompletion::Unsupported { .. } => RunStatus::Unsupported,
        ExtensionCompletion::Cancelled => RunStatus::Cancelled,
        ExtensionCompletion::ExceededBudget { .. } => RunStatus::ExceededBudget,
        ExtensionCompletion::Truncated { .. }
        | ExtensionCompletion::Complete
        | ExtensionCompletion::Ambiguous
        | ExtensionCompletion::Unknown
        | ExtensionCompletion::Unproven => RunStatus::Incomplete,
    }
}

fn source_inventory_digest(root: &Path) -> Result<StableDigest, TemplateError> {
    let mut files = Vec::new();
    collect_files(root, root, &mut files)?;
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let mut hasher = Sha256::new();
    hasher.update(b"bifrost-extension-template-source-inventory-v1\0");
    for (path, bytes) in files {
        hasher.update((path.len() as u64).to_le_bytes());
        hasher.update(path.as_bytes());
        hasher.update((bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
    }
    StableDigest::parse(format!("{:x}", hasher.finalize())).map_err(TemplateError::new)
}

fn collect_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<(String, Vec<u8>)>,
) -> Result<(), TemplateError> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| TemplateError::new(format!("read {}: {error}", directory.display())))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| TemplateError::new(error.to_string()))?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let file_type = entry
            .file_type()
            .map_err(|error| TemplateError::new(error.to_string()))?;
        if file_type.is_symlink() {
            return Err(TemplateError::new(
                "workspace fixture cannot contain symlinks",
            ));
        }
        if file_type.is_dir() {
            collect_files(root, &entry.path(), files)?;
        } else if file_type.is_file() {
            let relative = entry
                .path()
                .strip_prefix(root)
                .map_err(|error| TemplateError::new(error.to_string()))?
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            files.push((relative, read(&entry.path())?));
        }
    }
    Ok(())
}

fn strict_json<T: for<'de> Deserialize<'de>>(
    bytes: &[u8],
    label: &str,
) -> Result<T, TemplateError> {
    serde_json::from_slice(bytes)
        .map_err(|error| TemplateError::new(format!("decode {label}: {error}")))
}

fn canonicalize_json_bytes(bytes: &[u8]) -> Result<Vec<u8>, TemplateError> {
    let value: Value = strict_json(bytes, "JSON artifact")?;
    canonical_json(&value)
}

fn canonical_json(value: &impl Serialize) -> Result<Vec<u8>, TemplateError> {
    let value = serde_json::to_value(value)
        .map_err(|error| TemplateError::new(format!("serialize JSON: {error}")))?;
    let mut bytes = serde_json::to_vec(&canonical_value(value))
        .map_err(|error| TemplateError::new(format!("encode JSON: {error}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn canonical_value(value: Value) -> Value {
    match value {
        Value::Object(values) => {
            let mut entries = values.into_iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            Value::Object(
                entries
                    .into_iter()
                    .map(|(key, value)| (key, canonical_value(value)))
                    .collect::<Map<_, _>>(),
            )
        }
        Value::Array(values) => Value::Array(values.into_iter().map(canonical_value).collect()),
        scalar => scalar,
    }
}

fn sha256(bytes: &[u8]) -> StableDigest {
    StableDigest::parse(format!("{:x}", Sha256::digest(bytes))).expect("SHA-256 is valid")
}

fn read(path: &Path) -> Result<Vec<u8>, TemplateError> {
    fs::read(path).map_err(|error| TemplateError::new(format!("read {}: {error}", path.display())))
}
