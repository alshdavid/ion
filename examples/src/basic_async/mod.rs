// use ion::*;

pub fn main() -> anyhow::Result<()> {
    // let runtime = JsRuntime::initialize_once()?;

    // // Create an isolate running on a dedicated thread
    // let worker = runtime.spawn_worker()?;
    // let ctx = worker.create_context()?;

    // ctx.exec_blocking(|env| {
    //     // Spawn an future on the event loop
    //     env.spawn_local({
    //         let env = env.clone();
    //         async move {
    //             println!("Async Task Started");

    //             // Evaluate arbitrary JavaScript, the result of the last line is returned
    //             let value = env.eval_script::<JsNumber>("1 + 1")?;

    //             // Wait for some time
    //             tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    //             // Cast to Rust type
    //             let result = value.get_u32()?;
    //             println!("Async Task Returned: {}", result);

    //             Ok(())
    //         }
    //     })?;

    //     Ok(())
    // })?;

    Ok(())
}
