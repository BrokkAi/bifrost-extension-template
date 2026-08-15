use bifrost_extension_template::{
    AnalysisOptions, RunOptions, analyze_workspace, reproduce_bundle, run_lifecycle, verify_bundle,
};
use clap::{Parser, Subcommand};
use std::{path::PathBuf, process::ExitCode};

#[derive(Debug, Parser)]
#[command(about = "Build a static-analysis CLI on Bifrost's extension API")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Analyze {
        #[arg(long)]
        workspace: PathBuf,
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        observations: PathBuf,
    },
    RunExample {
        #[arg(long)]
        output: PathBuf,
    },
    Verify {
        #[arg(long)]
        bundle: PathBuf,
    },
    Reproduce {
        #[arg(long)]
        bundle: PathBuf,
        #[arg(long)]
        workspace: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
}

fn main() -> ExitCode {
    match execute(Cli::parse().command) {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn execute(command: Command) -> Result<String, Box<dyn std::error::Error>> {
    match command {
        Command::Analyze {
            workspace,
            config,
            observations,
        } => Ok(serde_json::to_string_pretty(&analyze_workspace(
            &AnalysisOptions {
                workspace,
                config,
                observations,
            },
        )?)?),
        Command::RunExample { output } => {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let summary = run_lifecycle(&RunOptions {
                workspace: root.join("fixtures/workspace"),
                config: root.join("fixtures/input/config.json"),
                observations: root.join("fixtures/input/observations.json"),
                output,
            })?;
            Ok(format!(
                "generation={} cold={} reopen={}",
                summary.generation, summary.cold_manifest, summary.reopen_manifest
            ))
        }
        Command::Verify { bundle } => {
            let digest = verify_bundle(&bundle)?;
            Ok(format!("verified {digest}"))
        }
        Command::Reproduce {
            bundle,
            workspace,
            output,
        } => {
            let result = reproduce_bundle(&bundle, &workspace, &output)?;
            Ok(format!("reproduced {}", result.reproduced_manifest))
        }
    }
}
