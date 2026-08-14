use bifrost_extension_template::{RunOptions, reproduce_bundle, run_lifecycle, verify_bundle};
use std::{env, path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    match execute(&args) {
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

fn execute(args: &[std::ffi::OsString]) -> Result<String, Box<dyn std::error::Error>> {
    let Some(command) = args.first().and_then(|value| value.to_str()) else {
        return Err("usage: bifrost-extension-template <run-example|verify|reproduce> ...".into());
    };
    match command {
        "run-example" => {
            let output = flag_path(&args[1..], "--output")?;
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
        "verify" => {
            let bundle = flag_path(&args[1..], "--bundle")?;
            let digest = verify_bundle(&bundle)?;
            Ok(format!("verified {digest}"))
        }
        "reproduce" => {
            let bundle = flag_path(&args[1..], "--bundle")?;
            let workspace = flag_path(&args[1..], "--workspace")?;
            let output = flag_path(&args[1..], "--output")?;
            let result = reproduce_bundle(&bundle, &workspace, &output)?;
            Ok(format!("reproduced {}", result.reproduced_manifest))
        }
        _ => Err(format!("unknown command: {command}").into()),
    }
}

fn flag_path(
    args: &[std::ffi::OsString],
    name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let position = args
        .iter()
        .position(|value| value == name)
        .ok_or_else(|| format!("missing {name} PATH"))?;
    args.get(position + 1)
        .map(PathBuf::from)
        .ok_or_else(|| format!("missing value after {name}").into())
}
