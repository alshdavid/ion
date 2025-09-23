use ion::*;

pub fn main() -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(main_async())
}

async fn main_async() -> anyhow::Result<()> {
        let runtime = JsRuntime::initialize_debug()?;

    let worker = runtime.spawn_worker()?;

    println!("[ctx0] Started");
    let ctx0 = worker.create_context()?;

    ctx0.exec(|env| {
        let value = env.eval_script::<JsNumber>("1 + 1")?;
        let result = value.get_u32()?;
        println!("[ctx1]: {}", result);
        Ok(())
    })?;

    ctx0.wait_blocking();

    Ok(())
}

// async fn main_async() -> anyhow::Result<()> {
//     let tracker = tokio_util::task::TaskTracker::new();

//     tokio::task::spawn({
//         let tracker = tracker.clone();
//         async move {
//             tracker.spawn(async { println!("ok") });
//         }
//     });

//     tracker.close();

//     tracker.spawn(async { println!("ok") });

//     tracker.wait().await;

//     Ok(())
// }

/*


pub fn main() -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(main_async())
}

async fn main_async() -> anyhow::Result<()> {
    let tracker = tokio_util::task::TaskTracker::new();

    tokio::task::spawn({
        let tracker = tracker.clone();
        async move {
            tracker.spawn(async { println!("ok") });
        }
    });

    tracker.close();
    tracker.wait().await;

    Ok(())
}


*/