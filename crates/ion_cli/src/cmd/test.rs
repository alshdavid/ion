use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use flume::unbounded;
use ion::utils::PathExt;
use ion::*;
use normalize_path::NormalizePath;
use tokio::task::JoinSet;

#[derive(Debug, Parser)]
pub struct TestCommand {
    /// Target get file to run
    pub files: Vec<PathBuf>,
}

pub fn main(command: TestCommand) -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus::get_physical())
        .enable_all()
        .build()?
        .block_on(main_async(command))
}

async fn main_async(command: TestCommand) -> anyhow::Result<()> {
    let mut entries = vec![];

    let Ok(cwd) = std::env::current_dir() else {
        return Err(anyhow::anyhow!("Unable to get cwd"));
    };

    // Convert paths from relative to absolute
    for file in command.files {
        if file.is_absolute() {
            entries.push(file.normalize());
        } else {
            entries.push(cwd.join(&file).normalize());
        }
    }

    let runtime = ion::JsRuntime::initialize_once(JsRuntimeOptions {
        v8_args: vec![],
        resolvers: vec![ion::resolvers::relative()],
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
    })?;

    let (tx, rx) = unbounded::<(PathBuf, String, u32)>();
    let mut set = JoinSet::<anyhow::Result<()>>::new();

    for _ in 0..1 {
        set.spawn({
            let runtime = Arc::clone(&runtime);
            let rx = rx.clone();
            async move {
                while let Ok((file, message, test_id)) = rx.recv() {
                    let worker = runtime.spawn_worker(JsWorkerOptions::default())?;
                    let ctx = worker.create_context()?;

                    println!("- {}", message);
                    ctx.import(file.try_to_string()?)?;
                    ctx.exec_async(move |env| {
                        env.eval_module(format!(
                            r#"
                            import {{ getTests }} from "ion:test"
                            const tests = getTests()
                            tests[{}][1]()
                        "#,
                            test_id
                        ))?;
                        Ok(())
                    })
                    .await?;
                }
                Ok(())
            }
        });
    }

    for file in entries {
        let worker = runtime.spawn_worker(JsWorkerOptions::default())?;
        let ctx = worker.create_context()?;

        ctx.import(file.try_to_string()?)?;
        ctx.exec({
            let tx = tx.clone();
            move |env| {
                let module = env.eval_module(
                    r#"
                import { getTests } from "ion:test"
                export default getTests()
            "#,
                )?;

                let tests = module.get_named_property_unchecked::<JsObject>("default")?;
                let length = tests.get_array_length()?;
                for i in 0..length {
                    let Some(value) = tests.get_element::<JsObject>(i)? else {
                        panic!();
                    };
                    let message = value.get_element::<JsString>(0)?.unwrap();
                    tx.try_send((file.clone(), message.get_string()?, i))?;
                }

                Ok(())
            }
        })?;
    }

    drop(tx);

    while let Some(res) = set.join_next().await {
        res??
    }
    Ok(())
}
