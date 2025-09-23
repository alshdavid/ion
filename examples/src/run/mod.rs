use std::path::PathBuf;

use ion::utils::PathExt;
use ion::*;
use normalize_path::NormalizePath;

pub fn main() -> anyhow::Result<()> {
    let file_path = std::env::args()
        .collect::<Vec<String>>()
        .get(2)
        .cloned()
        .expect("No filepath provided");

    let file_path = PathBuf::from(file_path);
    let file_path = if file_path.is_absolute() {
        file_path
    } else {
        let Ok(cwd) = std::env::current_dir() else {
            return Err(anyhow::anyhow!("Unable to get cwd"));
        };
        cwd.join(&file_path)
    }
    .normalize();

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

    let worker = runtime.spawn_worker(JsWorkerOptions::default())?;
    let ctx = worker.create_context()?;

    ctx.import(file_path.try_to_string()?)?;

    Ok(())
}
