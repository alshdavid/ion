use std::path::PathBuf;
use std::sync::Arc;

use ion::utils::PathExt;
use ion::*;

static CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub fn main() -> anyhow::Result<()> {
    let runtime = JsRuntime::initialize_once(JsRuntimeOptions {
        resolvers: vec![custom_resolver()],
        ..Default::default()
    })?;

    let worker = runtime.spawn_worker(JsWorkerOptions::default())?;
    let ctx = worker.create_context()?;

    let entry_point = PathBuf::from(CARGO_MANIFEST_DIR)
        .join("js")
        .join("modules")
        .join("index.js")
        .try_to_string()?;

    ctx.import(&entry_point)?;

    Ok(())
}

pub fn custom_resolver() -> JsResolver {
    Arc::new(|ctx: ResolverContext| -> JsResolverFut {
        Box::pin(async move {
            println!("Custom Resolver Has Run For Path {:?}", ctx.from);
            Ok(None)
        })
    })
}
