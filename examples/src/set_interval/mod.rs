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

    let worker = runtime.spawn_worker()?;
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
