use std::thread;
use std::time::Duration;

use ion::*;

use crate::testing::MemoryUsageCounter;

pub fn main() -> anyhow::Result<()> {
    let memu = MemoryUsageCounter::default();
    println!("[0] {:?}", memu);

    let runtime = JsRuntime::initialize_once(JsRuntimeOptions::default())?;
    println!("[1] {:?}", memu);

    let worker = runtime.spawn_worker()?;

    for i in 2..50 {
        let ctx = worker.create_context()?;
        let mut v = vec![];

        for _ in 2..1000 {
            let tsfn = ctx.exec_blocking(|env| {
                let func = JsFunction::new(env, |_env, ctx| ctx.arg::<JsNumber>(0))?;
                ThreadSafeFunction::new(&func)
            })?;

            tsfn.call_blocking(
                // Map Args
                |_env| Ok(42),
                // Map Ret
                move |_env, ret| ret.cast::<JsNumber>()?.get_u32(),
            )
            .unwrap();

            v.push(tsfn);
        }

        v.clear();
        worker.run_garbage_collection_for_testing()?;
        println!("[{}] {:?}", i, memu);
        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
