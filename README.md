# Ion

## JavaScript Runtime for Rust

Goals:
- High level API for v8
  - Inspired by napi-rs
- C bindings for v8
  - Inspired by Nodejs n-api

## CLI Usage

```bash
just build
./target/linux-amd64/ion eval "console.log('42')"
```

## Embedder Usage

### Basic

See (./examples)[./examples]

```rust
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
```

### Async

```rust
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
            // Spawn an future on the event loop
            env.spawn_async_local({
                let env = env.clone();
                async move {
                    // Open scope for execution (TODO hide this)
                    let scope = env.context_scope();

                    // Evaluate arbitrary JavaScript, the result of the last line is returned
                    let value = env.eval_script("1 + 1").unwrap();

                    // Cast to Rust type
                    let result = value.int32_value(scope).unwrap();

                    println!("Returned: {}", result);
                }
            })?;

            Ok(())
        })?;
    };

    Ok(())
}
```