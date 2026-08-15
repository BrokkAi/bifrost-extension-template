use bifrost_extension_template::{AnalysisOptions, analyze_workspace};
use rmcp::{
    ErrorData, ServiceExt,
    handler::server::wrapper::{Json, Parameters},
    schemars, tool, tool_router,
};
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AnalyzeParams {
    /// Workspace root to open as an immutable Bifrost generation.
    workspace: PathBuf,
    /// Extension-owned analysis configuration JSON file.
    config: PathBuf,
    /// Extension-owned generic observations JSON file.
    observations: PathBuf,
}

#[derive(Debug, Clone)]
struct AnalysisServer {
    adapter_name: &'static str,
}

impl Default for AnalysisServer {
    fn default() -> Self {
        Self {
            adapter_name: "bifrost-extension-mcp-example",
        }
    }
}

#[tool_router(server_handler)]
impl AnalysisServer {
    #[tool(
        name = "analyze_workspace",
        description = "Map observations and acquire bounded semantic relations from Bifrost"
    )]
    fn analyze(
        &self,
        Parameters(params): Parameters<AnalyzeParams>,
    ) -> Result<Json<Value>, ErrorData> {
        let result = analyze_workspace(&AnalysisOptions {
            workspace: params.workspace,
            config: params.config,
            observations: params.observations,
        })
        .map_err(|error| {
            ErrorData::invalid_params(format!("{}: {error}", self.adapter_name), None)
        })?;
        let value = serde_json::to_value(result)
            .map_err(|error| ErrorData::internal_error(error.to_string(), None))?;
        Ok(Json(value))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    AnalysisServer::default()
        .serve(rmcp::transport::stdio())
        .await?
        .waiting()
        .await?;
    Ok(())
}
