use std::time::Duration;

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

    // Create an isolate running on a dedicated thread
    let worker = runtime.spawn_worker()?;

    // // Open a JavaScript context on the isolate thread to execute JavaScript on
    // // You can open multiple contexts, sharing the same thread
    let ctx = worker.create_context()?;

    // Execute some JavaScript in the context
    ctx.exec(|env| {
        env.inc_ref();

        env.spawn_background({
            let env = env.as_async();
            async move {
                println!("Background Task Started");
                tokio::time::sleep(Duration::from_secs(1)).await;
                println!("Background Task Ended");

                env.exec_async(|env| {
                    println!("hi");
                    env.dec_ref();
                    Ok(())
                })
                .await?;
                Ok(())
            }
        })?;

        Ok(())
    })?;

    println!("Context Dropping");
    drop(ctx);
    drop(worker);

    println!("Context Dropped");

    Ok(())
}
