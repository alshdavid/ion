use ion::*;

pub fn main() -> anyhow::Result<()> {
    let code = std::env::args()
        .collect::<Vec<String>>()
        .get(2)
        .cloned()
        .expect("No code provided");

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

    ctx.eval(code)?;
    ctx.join()?;

    Ok(())
}
