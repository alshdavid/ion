use ion::*;

pub fn main() -> anyhow::Result<()> {
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

    // Set External Value
    ctx.exec_blocking(|env| {
        let external = 42;
        let external_js = JsExternal::new(env, external)?;
        env.global_this()?
            .set_named_property("external", external_js)?;
        Ok(())
    })?;

    // Use External Value
    ctx.exec_blocking(|env| {
        let Some(external_js) = env
            .global_this()?
            .get_named_property::<JsExternal<i32>>("external")?
        else {
            panic!("Could not revive external")
        };
        let external = external_js.as_inner()?;
        println!("{}", external);
        Ok(())
    })?;

    ctx.join()?;
    Ok(())
}
