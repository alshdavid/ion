# Ion

## JavaScript Runtime for Rust

High level API that wraps V8 and exposes an n-api inspired API for interacting with.


```rust
fn main() -> anyhow::Result<()> {
  // Spawn a JavaScript isolate on it's own thread
  let ctx = ion::Context::new()?;

  // Interact with that isolate
  let result: u32 = ctx.exec_blocking(|env| {
    // Load some code into the isolate
    env.eval("globalThis.handler = (a, b) => a + b")?;
    
    let a: ion::JsNumber = env.create_uint32(1)?;
    let b: ion::JsNumber = env.create_uint32(1)?;

    let global_this: ion::JsObject = env.global_this()?;
    let handler: ion::JsFunction = global_this.get_property("handler")?;

    let result: ion::JsNumber = handler.call_with_args(None, &[a, b])?;
    result.to_u32()
  })?;

  println!("{}", result); // "2"
  Ok(())
}
```