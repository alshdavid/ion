use std::path::PathBuf;

use ion::utils::PathExt;
use ion::*;

static CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub fn main() -> anyhow::Result<()> {
    let entry_point = PathBuf::from(CARGO_MANIFEST_DIR)
        .join("src")
        .join("transformers")
        .join("js")
        .join("main.js");

    let runtime = JsRuntime::initialize_once(JsRuntimeOptions {
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
        ..Default::default()
    })?;

    let worker = runtime.spawn_worker(JsWorkerOptions::default())?;
    let ctx = worker.create_context()?;

    ctx.exec_blocking(move |env| env.import(entry_point.try_to_string()?))?;

    ctx.join()?;
    Ok(())
}
