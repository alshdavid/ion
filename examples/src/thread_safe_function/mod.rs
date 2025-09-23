use std::thread;
use std::time::Duration;

use ion::*;

pub fn main() -> anyhow::Result<()> {
    let rt = JsRuntime::initialize_once(JsRuntimeOptions {
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

    let wrk = rt.spawn_worker()?;
    let ctx = wrk.create_context()?;

    ctx.exec_blocking(|env| {
        let func = JsFunction::new(env, |_env, ctx| {
            let arg0 = ctx.arg::<JsNumber>(0)?;
            let arg1 = ctx.arg::<JsNumber>(1)?;

            let result = arg0.get_u32()? + arg1.get_u32()?;
            Ok(result)
        })?;

        let tsfn = ThreadSafeFunction::new(&func)?;

        thread::spawn({
            let tsfn = tsfn.clone();
            move || {
                let a = 1;
                let b = 1;

                let ret = tsfn
                    .call_blocking(
                        // Rust values to pass into JavaScript
                        move |_env| Ok((a, b)),
                        // JavaScript values to pass back into Rust
                        |_env, ret| ret.cast::<JsNumber>()?.get_u32(),
                    )
                    .unwrap();

                println!("Ret: {}", ret);
                thread::sleep(Duration::from_secs(1));
            }
        });

        thread::spawn({
            let tsfn = tsfn.clone();
            move || {
                let a = 1;
                let b = 1;

                let ret = tsfn
                    .call_blocking(
                        // Rust values to pass into JavaScript
                        move |_env| Ok((a, b)),
                        // JavaScript values to pass back into Rust
                        |_env, ret| ret.cast::<JsNumber>()?.get_u32(),
                    )
                    .unwrap();

                println!("Ret: {}", ret);
                thread::sleep(Duration::from_secs(1));
            }
        });

        Ok(())
    })?;

    Ok(())
}
