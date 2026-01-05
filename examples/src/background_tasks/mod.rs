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
    let worker = runtime.spawn_worker(JsWorkerOptions::default())?;

    // // Open a JavaScript context on the isolate thread to execute JavaScript on
    // // You can open multiple contexts, sharing the same thread
    let ctx = worker.create_context()?;

    // Execute some JavaScript in the context
    ctx.exec_blocking(|env| {
        // Tell the runtime that it must remain alive
        // Safety: This must be manually incremented and decremented
        //         if the ref count is greater than 0, the runtime
        //         will remain alive until it's 0
        env.inc_ref();

        env.spawn_background({
            let env = env.as_async();
            async move {
                println!("Task [rs]: Started");
                tokio::time::sleep(Duration::from_secs(1)).await;

                env.exec_async(|env| {
                    println!("Task [js]: Message");
                    // Tell the runtime that this task will no longer require keeping it alive
                    env.dec_ref();
                    Ok(())
                })
                .await?;

                println!("Task [rs]: Ended");
                Ok(())
            }
        })?;

        Ok(())
    })?;

    ctx.join()?;

    Ok(())
}
