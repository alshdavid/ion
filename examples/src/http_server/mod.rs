mod handler;
mod http1;
mod request;
mod response;
mod worker_pool;

use std::sync::Arc;

use ion::JsRuntime;
use tokio::io::AsyncWriteExt;

use self::http1::ResponseBuilderExt;
use self::worker_pool::WorkerPool;

const HANDLER: &str = include_str!("../../js/faas-handlers/index.handler.js");

#[derive(Debug)]
pub enum HttpEvent {
    WriteHead(u32),
    Write(String),
    End,
}

pub fn main() -> anyhow::Result<()> {
    // Start the runtime from the main thread
    let runtime = ion::platform::initialize_once()?;

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(num_cpus::get_physical())
        .build()
        .unwrap()
        .block_on(main_async(runtime))
}

async fn main_async(runtime: JsRuntime) -> anyhow::Result<()> {
    // Spawn a pool of JavaScript worker threads. Load balance with round-robin
    let workers = Arc::new(WorkerPool::new(&runtime, num_cpus::get_physical()));

    http1::http1_server("0.0.0.0:4200", move |_req, res| {
        let workers = workers.clone();
        async move {
            // Spawn a JavaScript context on one of the worker threads
            let ctx = workers.create_context();

            // Channel to communicate with the JavaScript thread
            let (tx, rx) = ion::utils::channel::channel::<HttpEvent>();

            // Execute javascript on the context thread (non blocking)
            // Pass in a channel to get calls to the response
            ctx.exec(move |env| {
                // Initialize Globals
                ion::exts::define_console(&env);
                ion::exts::define_set_interval(&env);
                ion::exts::define_set_timeout(&env);

                // Run Handler Script
                env.eval_script(HANDLER)?;

                // Open scope to execute handler
                let scope = &mut env.open_scope();

                // Get handler from globalThis
                // TODO use module exports
                let js_handler = handler::get_handler(scope);

                // Construct req/res types
                let http_request = request::v8_create_http_request(scope);
                let http_response = response::v8_create_http_response(scope, &tx);

                // Execute handler
                // TODO promises
                let recv = v8::undefined(scope);
                js_handler.call(scope, recv.into(), &[http_request, http_response]);

                Ok(())
            })?;

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
