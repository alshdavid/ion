use std::path::PathBuf;

use ion::utils::PathExt;
use ion::*;

static CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub fn main() -> anyhow::Result<()> {
    let entry_point = PathBuf::from(CARGO_MANIFEST_DIR)
        .join("src")
        .join("testing")
        .join("typescript")
        .join("js")
        .join("main.ts");

    let runtime = JsRuntime::initialize_once(JsRuntimeOptions {
        extensions: vec![
            ion::extensions::console(),
            ion::extensions::set_interval(),
            ion::extensions::set_timeout(),
        ],
        transformers: vec![
            ion::transformers::json(),
            ion::transformers::ts(),
            ion::transformers::tsx(),
        ],
        ..Default::default()
    })?;

    let worker = runtime.spawn_worker()?;
    let ctx = worker.create_context()?;

    ctx.exec_blocking(move |env| env.import(entry_point.try_to_string()?))?;

    Ok(())
}
