use std::thread;
use std::time::Duration;

use ion::*;

pub fn main() -> anyhow::Result<()> {
    let rt = JsRuntime::initialize_once(JsRuntimeOptions {
        extensions: vec![ion::extensions::console()],
        ..Default::default()
    })?;

    let wrk = rt.spawn_worker()?;
    let ctx = wrk.create_context()?;

    ctx.exec_blocking(|env| {
        let (promise, deferred) = JsDeferred::new(env)?;

        thread::spawn({
            move || {
                thread::sleep(Duration::from_secs(1));
                deferred.resolve(|_env| {
                    //
                    Ok(42)
                })
            }
        });

        let mut global_this = env.global_this()?;
        global_this.set_named_property("foo", promise)?;

        Ok(())
    })?;

    ctx.eval(
        r#"
        globalThis.foo
            .then(() => console.log("Done"))
            .catch(() => console.log("threw"))        
    "#,
    )?;

    Ok(())
}
