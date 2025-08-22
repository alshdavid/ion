pub fn main() -> anyhow::Result<()> {
    // Start the runtime
    let runtime = ion::platform::initialize_once()?;

    // Create an isolate running on a dedicated thread
    let worker = runtime.spawn_worker()?;

    // Open a JavaScript context on the isolate thread to execute JavaScript on
    // You can open multiple contexts, sharing the same thread
    {
        let ctx = worker.create_context()?;

        // Execute some JavaScript in the context
        ctx.exec_blocking(|env| {
            // Open scope for execution (TODO hide this)
            let scope = env.context_scope();

            // Evaluate arbitrary JavaScript, the result of the last line is returned
            let value = env.eval_script("1 + 1")?;

            // Cast to Rust type
            let result = value.int32_value(scope).unwrap();

            println!("Returned: {}", result);
            Ok(())
        })?;
    };

    Ok(())
}
