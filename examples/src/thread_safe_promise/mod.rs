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

    let worker = runtime.spawn_worker(JsWorkerOptions::default())?;
    let ctx = worker.create_context()?;

    // Execute some JavaScript in the context
    let promise = ctx.exec_blocking(|env| {
        // Evaluate arbitrary JavaScript, the result of the last line is returned
        let value = env.eval_script::<JsPromise>(
            r#"
            console.log("[JS] Promise Started");

            new Promise((resolve) => setTimeout(() => {
                console.log("[JS] Promise Resolved");
                resolve(42);
            }, 1000));
        "#,
        )?;

        println!("Exec Complete (Not Blocked)");
        ThreadSafePromise::new(&value)
    })?;

    // Cast to Rust type
    let result = promise.settled_blocking::<JsNumber, _>(|_env, result| match result {
        JsPromiseResult::Resolved(resolved) => resolved.get_u32(),
        JsPromiseResult::Rejected(_) => unreachable!(),
    })?;

    println!("[Rust] Got {}", result);

    ctx.join()?;
    Ok(())
}
