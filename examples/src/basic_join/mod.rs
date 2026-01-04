use std::time::Duration;

use ion::*;

pub fn main() -> anyhow::Result<()> {
    let runtime = JsRuntime::initialize_once(JsRuntimeOptions::default())?;

    // Create an isolate running on a dedicated thread
    let worker = runtime.spawn_worker(JsWorkerOptions::default())?;

    // // Open a JavaScript context on the isolate thread to execute JavaScript on
    // // You can open multiple contexts, sharing the same thread
    let ctx = worker.create_context()?;

    // Execute some JavaScript in the context
    ctx.exec(|env| {
        env.inc_ref();
        println!("Async Start");

        env.spawn_background({
            let env = env.as_async();
            async move {
                tokio::time::sleep(Duration::from_millis(1000)).await;
                env.exec_async(|env| {
                    env.dec_ref();
                    println!("Async Done");
                    Ok(())
                })
                .await
            }
        })?;
        Ok(())
    })?;

    // Wait for context to complete
    // ctx.join_blocking()?;

    // Wait for all contexts within worker to complete
    worker.join_blocking()?;
    println!("Fin");

    Ok(())
}
