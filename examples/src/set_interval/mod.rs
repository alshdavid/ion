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

    ctx.exec_blocking(|env| {
        env.eval_script::<JsUnknown>(
            r#"
                let i = 0;

                let timerRef = setInterval(() => {
                    console.log(`${i} Interval Ran`);
                    i += 1;
                }, 100);

                setTimeout(() => {
                    console.log(`setInterval cleared`);
                    clearInterval(timerRef);
                }, 500);
            "#,
        )?;

        Ok(())
    })?;
    Ok(())
}
