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

    // Execute some JavaScript in the context
    let promise = ctx.exec_blocking(|env| {
        // Evaluate arbitrary JavaScript, the result of the last line is returned
        let value = env.eval_script::<JsPromise>(
            r#"
            console.log("[JS] Promise Started");

            new Promise((resolve) => setTimeout(() => {
                console.log("[JS] Promise Resolved");
                resolve(42);
            }, 3_000));
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

    Ok(())
}
