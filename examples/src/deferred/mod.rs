use std::thread;
use std::time::Duration;

use ion::*;

pub fn main() -> anyhow::Result<()> {
    let runtime = JsRuntime::initialize_once(JsRuntimeOptions {
        transformers: vec![
            ion::transformers::json(),
            ion::transformers::ts(),
            ion::transformers::tsx(),
        ],
        extensions: vec![
            ion::extensions::event_target(),
            ion::extensions::console(),
            ion::extensions::set_timeout(),
            ion::extensions::set_interval(),
            ion::extensions::test(),
            ion::extensions::global_this(),
        ],
        ..Default::default()
    })?;
    let wrk = runtime.spawn_worker(JsWorkerOptions::default())?;
    let ctx = wrk.create_context()?;

    ctx.exec_blocking(|env| {
        let (promise, deferred) = JsDeferred::new(env)?;

        thread::spawn({
            move || {
                println!("Promise Start");
                thread::sleep(Duration::from_secs(1));
                
                deferred.resolve(|_env| {
                    //
                    Ok(42)
                }).expect("Deferred to complete");

                println!("Promise End");
            }
        });

        let mut global_this = env.global_this()?;
        global_this.set_named_property("foo", promise)?;

        Ok(())
    })?;

    ctx.exec_blocking(|env| {
        env.eval_script::<JsUnknown>(r#"
            console.log("Eval: Start");
            
            globalThis.foo
                .then((value) => console.log(`Done: ${value}`))
                .catch(() => console.log("threw"));

            console.log("Eval: End");     
        "#)?;
        Ok(())
    })?;

    Ok(())
}
