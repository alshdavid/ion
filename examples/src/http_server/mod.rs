mod handlers;
mod http1;
mod worker_pool;

use std::sync::Arc;

use ion::utils::channel::channel;
use ion::*;
use tokio::io::AsyncWriteExt;

use crate::http_server::handlers::get_handler;
use crate::http_server::worker_pool::WorkerPoolOptions;

use self::http1::ResponseBuilderExt;
use self::worker_pool::WorkerPool;

pub fn main() -> anyhow::Result<()> {
    // Start the runtime from the main thread
    let runtime = JsRuntime::initialize_debug()?;

    // Register extensions
    runtime.register_extension(ion::extensions::console())?;
    runtime.register_extension(ion::extensions::set_timeout())?;
    runtime.register_extension(ion::extensions::set_interval())?;

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(num_cpus::get_physical())
        .build()
        .unwrap()
        .block_on(main_async(runtime))
}

async fn main_async(runtime: Arc<JsRuntime>) -> anyhow::Result<()> {
    // Spawn a pool of JavaScript worker threads. Load balance with round-robin
    let workers = Arc::new(WorkerPool::new(WorkerPoolOptions {
        runtime,
        // How many threads to create
        worker_count: num_cpus::get_physical(),
        // How many JavaScript contexts to share that thread
        contexts_per_worker: num_cpus::get_physical(),
    })?);

    http1::http1_server("0.0.0.0:4200", move |req, res| {
        let workers = workers.clone();
        async move {
            let route = get_handler(req.uri().path()).await?;

            // Convert JavaScript handler into a ThreadSafe Function
            let handler = workers
                .exec_async(move |env| {
                    // Run Handler Script
                    let module = env.eval_module(route)?;
                    let js_handler =
                        module.get_named_property_unchecked::<JsFunction>("handler")?;
                    ThreadSafeFunction::new(&js_handler)
                })
                .await?;

            let (tx, rx) = channel::<HttpEvent>();

            // Non blocking call to function
            handler.call(
                // Map the req/res types
                move |env| {
                    let http_request = JsObject::new(env)?;

                    let mut http_response = JsObject::new(env)?;

                    let write_head = JsFunction::new(env, {
                        let tx = tx.clone();
                        move |env, ctx| {
                            let arg0 = ctx.arg::<JsNumber>(0)?;
                            tx.try_send(HttpEvent::WriteHead(arg0.get_u32()?))?;
                            env.get_undefined()
                        }
                    })?;

                    let write = JsFunction::new(env, {
                        let tx = tx.clone();
                        move |env, ctx| {
                            let arg0 = ctx.arg::<JsString>(0)?;
                            tx.try_send(HttpEvent::Write(arg0.get_string()?))?;
                            env.get_undefined()
                        }
                    })?;

                    let end = JsFunction::new(env, {
                        let tx = tx.clone();
                        move |env, _ctx| {
                            tx.try_send(HttpEvent::End)?;
                            env.get_undefined()
                        }
                    })?;

                    http_response.set_named_property("writeHead", write_head)?;
                    http_response.set_named_property("write", write)?;
                    http_response.set_named_property("end", end)?;

                    // Pass arguments to handler
                    Ok((http_request, http_response))
                },
                // No return value
                thread_safe_function::map_return::noop,
            )?;

            // Construct streamed response
            let mut res = res.status(200);
            let mut buf = Vec::<u8>::new();

            // Buffer body until status code is sent (blocking)
            while let Ok(event) = rx.recv_async().await {
                match event {
                    HttpEvent::WriteHead(status) => {
                        res = res.status(status as u16);
                        break;
                    }
                    HttpEvent::Write(body) => {
                        buf.extend(body.as_bytes());
                    }
                    HttpEvent::End => {
                        panic!("Cannot end early")
                    }
                };
            }

            // Create a body stream
            let (res, mut writer) = res.body_stream(1)?;

            // Spawn a concurrent task to handle async writes to body (non blocking)
            tokio::task::spawn(async move {
                writer.write_all(&buf).await.unwrap();

                while let Ok(event) = rx.recv_async().await {
                    match event {
                        HttpEvent::WriteHead(_status) => {
                            panic!("Cannot send status twice")
                        }
                        HttpEvent::Write(body) => {
                            writer.write_all(body.as_bytes()).await.unwrap();
                        }
                        // TODO automatically end the handler
                        // Relying on GC to clean up res thereby dropping the Sender is unreliable.
                        // Options to consider, have the handler return a promise that drops the
                        // Sender when resolved. This would ignore any async js tasks in the background.
                        HttpEvent::End => break,
                    };
                }
            });

            // Return response
            Ok(res)
        }
    })
    .await?;

    Ok(())
}

#[derive(Debug)]
pub enum HttpEvent {
    WriteHead(u32),
    Write(String),
    End,
}
