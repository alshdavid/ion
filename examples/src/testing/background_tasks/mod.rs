// use std::{sync::atomic::{AtomicUsize, Ordering}, thread, time::Duration};

// use ion::{platform::background_worker::BackgroundTaskManager, *};

// pub fn main() -> anyhow::Result<()> {
//     let runtime = JsRuntime::initialize_once()?;

//     // Create an isolate running on a dedicated thread
//     let worker = runtime.spawn_worker()?;

//     // // Open a JavaScript context on the isolate thread to execute JavaScript on
//     // // You can open multiple contexts, sharing the same thread
//     let ctx = worker.create_context()?;

//     // Execute some JavaScript in the context
//     ctx.exec(|env| {
//         env.inc_ref();

//         env.spawn_background(|env| Box::pin(async move {
//             println!("Background Task Started");
//             tokio::time::sleep(Duration::from_secs(1)).await;
//             println!("Background Task Ended");

//             env.exec_async(|env| {
//                 println!("hi");
//                 env.dec_ref();
//                 Ok(())
//             }).await?;
//             Ok(())
//         }))?;

//         Ok(())
//     })?;

//     println!("Context Dropping");
//     drop(ctx);
//     drop(worker);

//     println!("Context Dropped");

//     thread::sleep(Duration::from_secs(2));

//     Ok(())
// }

use std::thread;
use std::time::Duration;

use ion::*;

pub fn main() -> anyhow::Result<()> {
    let rt = JsRuntime::initialize_once()?;
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
                        move |_env| {
                            Ok((a, b))
                        },
                        // JavaScript values to pass back into Rust
                        |_env, ret| {
                            
                            ret.cast::<JsNumber>()?.get_u32()
                        },
                    )
                    .unwrap();

                thread::sleep(Duration::from_secs(1));
                println!("Ret1: {}", ret);
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
                        move |_env| {
                            Ok((a, b))
                        },
                        // JavaScript values to pass back into Rust
                        |_env, ret| {
                            
                            ret.cast::<JsNumber>()?.get_u32()
                        },
                    )
                    .unwrap();

                thread::sleep(Duration::from_secs(2));
                println!("Ret2: {}", ret);
            }
        });

        Ok(())
    })?;

    Ok(())
}
