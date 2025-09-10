use std::time::Duration;

use ion::*;

pub fn main() -> anyhow::Result<()> {
    let runtime = JsRuntime::initialize_once()?;

    // Create an isolate running on a dedicated thread
    let worker = runtime.spawn_worker()?;

    // // Open a JavaScript context on the isolate thread to execute JavaScript on
    // // You can open multiple contexts, sharing the same thread
    let ctx = worker.create_context()?;

    // Execute some JavaScript in the context
    ctx.exec(|env| {
        env.inc_ref();

        env.spawn_background(|env| {
            Box::pin(async move {
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
            })
        })?;

        Ok(())
    })?;

    println!("Context Dropping");
    drop(ctx);
    drop(worker);

    println!("Context Dropped");

    Ok(())
}
