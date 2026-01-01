use ion::*;

pub fn main() -> anyhow::Result<()> {
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

    let wrk1 = runtime.spawn_worker(JsWorkerOptions::default())?;
    let wrk2 = runtime.spawn_worker(JsWorkerOptions::default())?;
    let wrk3 = runtime.spawn_worker(JsWorkerOptions::default())?;

    let wrk1ctx1 = wrk1.create_context()?;
    let wrk2ctx1 = wrk2.create_context()?;
    let wrk3ctx1 = wrk3.create_context()?;

    wrk1ctx1.eval("console.log('wrk1ctx1')")?;
    wrk2ctx1.eval("console.log('wrk2ctx1')")?;
    wrk3ctx1.eval("console.log('wrk3ctx1')")?;

    Ok(())
}
