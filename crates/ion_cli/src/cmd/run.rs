use std::path::PathBuf;

use clap::Parser;
use ion::utils::PathExt;
use ion::*;
use normalize_path::NormalizePath;

#[derive(Debug, Parser)]
pub struct RunCommand {
    /// Target get file to run
    pub path: PathBuf,
}

pub fn main(command: RunCommand) -> anyhow::Result<()> {
    let entry = if command.path.is_absolute() {
        command.path
    } else {
        let Ok(cwd) = std::env::current_dir() else {
            return Err(anyhow::anyhow!("Unable to get cwd"));
        };
        cwd.join(&command.path).normalize()
    }
    .normalize();

    let runtime = JsRuntime::initialize_once(JsRuntimeOptions {
        v8_args: vec![],
        resolvers: vec![ion::resolvers::relative()],
        transformers: vec![
            ion::transformers::json(),
            ion::transformers::ts(),
            ion::transformers::tsx(),
        ],
        extensions: vec![
            ion::extensions::event_target(),
            ion::extensions::console(),
            ion::extensions::set_timeout(),
            ion::extensions::set_interval(),
            ion::extensions::test(),
            ion::extensions::global_this(),
        ],
    })?;

    let worker = runtime.spawn_worker(JsWorkerOptions::default())?;
    let ctx = worker.create_context()?;

    ctx.import(entry.try_to_string()?)?;
    Ok(())
}
