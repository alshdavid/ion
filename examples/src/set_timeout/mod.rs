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

    ctx.exec_blocking(|env| {
        env.eval_script::<JsUnknown>(
            r#"
                const sleep = d => new Promise(r => setTimeout(r, d))

                void async function main() {
                    console.log(`1`)
                    await sleep(1000)
                    console.log(`2`)
                    await sleep(1000)
                    console.log(`3`)
                    await sleep(1000)
                    console.log(`4`)
                    await sleep(1000)
                    console.log(`5`)
                }()
            "#,
        )?;

        Ok(())
    })?;
    Ok(())
}
