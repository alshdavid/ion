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
    ctx.exec_blocking(|env| {
        // Evaluate arbitrary JavaScript, the result of the last line is returned
        let value = env.eval_script::<JsPromise>(
            r#"
            new Promise((resolve) => {
                console.log("[JS] Promise Started");

                setTimeout(() => {
                    console.log("[JS] Promise Resolved");
                    resolve(42);
                }, 1000)
            });
        "#,
        )?;

        // Cast to Rust type
        value.settled::<JsNumber>(|_env, result| {
            match result {
                JsPromiseResult::Resolved(resolved) => {
                    println!("Resolved with: {}", resolved.get_u32()?)
                }
                JsPromiseResult::Rejected(err) => {
                    println!("Rejected With: {}", err.type_of())
                }
            };
            Ok(())
        })?;

        println!("Exec Complete (Not Blocked)");

        Ok(())
    })?;

    ctx.join()?;
    Ok(())
}
