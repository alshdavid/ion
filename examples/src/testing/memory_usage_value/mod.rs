use std::thread;
use std::time::Duration;

use ion::*;

use crate::testing::MemoryUsageCounter;

pub fn main() -> anyhow::Result<()> {
    let memu = MemoryUsageCounter::default();
    println!("[0] {:?}", memu);

    let runtime = JsRuntime::initialize_debug()?;
    println!("[1] {:?}", memu);

    let worker = runtime.spawn_worker()?;
    let ctx = worker.create_context()?;

    for i in 2..50 {
        for _ in 2..100 {
            let _value = ctx.exec_blocking(|env| {
                let value = env.eval_script::<JsNumber>("1 + 1")?;
                value.get_u32()
            })?;
        }

        println!("[{}] {:?}", i, memu);
        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
