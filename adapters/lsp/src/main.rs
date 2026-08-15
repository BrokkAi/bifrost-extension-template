use bifrost_extension_template::{AnalysisOptions, analyze_workspace};
use lsp_server::{Connection, Message, Request, Response};
use lsp_types::{ExecuteCommandOptions, ServerCapabilities, WorkDoneProgressOptions};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{error::Error, path::PathBuf};

const ANALYZE_COMMAND: &str = "bifrost-extension.analyze";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AnalyzeParams {
    workspace: PathBuf,
    config: PathBuf,
    observations: PathBuf,
}

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let (connection, io_threads) = Connection::stdio();
    let capabilities = ServerCapabilities {
        execute_command_provider: Some(ExecuteCommandOptions {
            commands: vec![ANALYZE_COMMAND.to_owned()],
            work_done_progress_options: WorkDoneProgressOptions::default(),
        }),
        ..ServerCapabilities::default()
    };
    let initialize = connection.initialize(serde_json::to_value(capabilities)?)?;
    let _: lsp_types::InitializeParams = serde_json::from_value(initialize)?;

    for message in &connection.receiver {
        if let Message::Request(request) = message {
            if connection.handle_shutdown(&request)? {
                break;
            }
            connection.sender.send(Message::Response(handle(request)))?;
        }
    }
    io_threads.join()?;
    Ok(())
}

fn handle(request: Request) -> Response {
    if request.method != "workspace/executeCommand" {
        return Response::new_err(request.id, -32_601, "method not found".to_owned());
    }
    let params = serde_json::from_value::<lsp_types::ExecuteCommandParams>(request.params)
        .and_then(|params| {
            if params.command != ANALYZE_COMMAND {
                return Err(serde::de::Error::custom("unknown analysis command"));
            }
            let value = params
                .arguments
                .into_iter()
                .next()
                .ok_or_else(|| serde::de::Error::custom("missing command argument"))?;
            serde_json::from_value::<AnalyzeParams>(value)
        });
    match params {
        Ok(params) => match analyze_workspace(&AnalysisOptions {
            workspace: params.workspace,
            config: params.config,
            observations: params.observations,
        }) {
            Ok(result) => Response::new_ok(
                request.id,
                serde_json::to_value(result).unwrap_or(Value::Null),
            ),
            Err(error) => Response::new_err(request.id, -32_603, error.to_string()),
        },
        Err(error) => Response::new_err(
            request.id,
            -32_602,
            json!({ "message": error.to_string() }).to_string(),
        ),
    }
}
