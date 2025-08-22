use std::{str::FromStr, sync::Arc};

use http::{HeaderName, HeaderValue, StatusCode};
use tokio::io::{AsyncWriteExt, DuplexStream};

use crate::{
    http1::ResponseBuilderExt,
    jsrt::{ContextPool, ContextPoolOptions, Environment},
};

mod http1;
mod js;
mod jsrt;
mod napi;

const SCRIPT: &str = include_str!("../handlers/hello-world.js");

fn main() -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(num_cpus::get_physical())
        .build()
        .unwrap()
        .block_on(main_async())
}

async fn main_async() -> anyhow::Result<()> {
    let pool = Arc::new(ContextPool::new(&ContextPoolOptions { threads: 8 }));

    http1::http1_server("0.0.0.0:8080", move |_req, res| {
        let pool = pool.clone();
        async move {
            // Handle synchronous events
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<HttpSyncEvents>();
            let tx = Chan(tx);
            // Streamed body

            pool.exec(move |mut env| {
                // Evaluate FaaS Function
                eval_script(&mut env, SCRIPT);

                // Get the exported handler
                let jsv_fn_handler = get_handler(&mut env);

                // Create Rust <-> JavaScript bindings
                let jsv_http_response = v8_create_http_response(&mut env.context_scope, tx)
                    .try_cast::<v8::Value>()
                    .expect("Cast to value");

                // Call FaaS handler
                let jsv_ukn_return = v8::undefined(&mut env.context_scope);
                jsv_fn_handler.call(
                    &mut env.context_scope,
                    jsv_ukn_return.into(),
                    &[jsv_http_response, jsv_http_response],
                );

                println!("finished handler")
            });

            let (mut res, mut writer) = res.body_stream(1)?;

            while let Some(ev) = rx.recv().await {
                match ev {
                    HttpSyncEvents::HeadersAppend((key, value)) => {
                        res.headers_mut().append(
                            HeaderName::from_str(key.as_str()).unwrap(),
                            HeaderValue::from_str(value.as_str()).unwrap(),
                        );
                    }
                    HttpSyncEvents::WriteHead(status) => {
                        (*res.status_mut()) = StatusCode::from_u16(status).unwrap();
                        break;
                    }
                    HttpSyncEvents::WriteBody(bytes) => {
                      println!("got bytes1");
                        writer.write_all(&bytes).await.unwrap();
                    }
                }
            }

            tokio::task::spawn(async move {
                while let Some(ev) = rx.recv().await {
                    let HttpSyncEvents::WriteBody(bytes) = ev else {
                        continue;
                    };
                      println!("got bytes2 {:?}", bytes);

                    writer.write_all(&bytes).await.unwrap();
                    break;
                  }
                  drop(writer)
            });

            println!("done");

            Ok(res)
        }
    })
    .await?;

    Ok(())
}

struct Chan(tokio::sync::mpsc::UnboundedSender<HttpSyncEvents>);

impl Clone for Chan {
    fn clone(&self) -> Self {
      println!("clone");
      Chan(self.0.clone())
    }
}

impl Drop for Chan {
    fn drop(&mut self) {
        println!("ded")
    }
}

enum HttpSyncEvents {
    HeadersAppend((String, String)),
    WriteHead(u16),
    WriteBody(Vec<u8>),
}

fn v8_create_http_response<'a>(
    scope: &mut v8::ContextScope<'a, v8::HandleScope>,
    sync_events: Chan,
) -> v8::Local<'a, v8::Object> {
    let jsv_obj_response = v8::Object::new(scope);

    let jsv_fn_write = js::v8_create_function_from_closure(scope, {
        let sync_events = sync_events.clone();
        move |scope: &mut v8::HandleScope,
              args: v8::FunctionCallbackArguments,
              _rv: v8::ReturnValue| {
            let value = args.get(0);
            let js_bytes = value.try_cast::<v8::String>().unwrap();
            let v = js_bytes.to_rust_string_lossy(scope);
            // js_bytes.copy_contents(&mut bytes);
            sync_events.0.send(HttpSyncEvents::WriteBody(v.into_bytes())).unwrap();
        }
    });
    js::v8_set_obj_property(scope, &jsv_obj_response, "write", jsv_fn_write.into());

    let jsv_fn_write_head = js::v8_create_function_from_closure(
        scope,
        move |scope: &mut v8::HandleScope,
              args: v8::FunctionCallbackArguments,
              _rv: v8::ReturnValue| {
            let value = args.get(0);
            let Some(status) = value.uint32_value(scope) else {
                panic!("NaN")
            };
            sync_events.0
                .send(HttpSyncEvents::WriteHead(status.try_into().unwrap()))
                .unwrap();
        },
    );
    js::v8_set_obj_property(
        scope,
        &jsv_obj_response,
        "writeHead",
        jsv_fn_write_head.into(),
    );

    jsv_obj_response
}

fn eval_script(env: &mut Environment, code: &str) {
    let code = v8::String::new(&mut env.context_scope, code).expect("Script to be valid string");

    let script = v8::Script::compile(&mut env.context_scope, code, None)
        .expect("Script to compile correctly");

    script
        .run(&mut env.context_scope)
        .expect("Script to run correctly");
}

fn get_handler<'a>(env: &mut Environment<'_, 'a>) -> v8::Local<'a, v8::Function> {
    let global_this = env.context.global(&mut env.context_scope);

    let jsv_obj_exports = js::v8_get_obj_property(&mut env.context_scope, &global_this, "exports")
        .expect("Handler to have globalThis.exports");

    let jsv_fn_handler = jsv_obj_exports
        .try_cast::<v8::Function>()
        .expect("Handler to be function");

    jsv_fn_handler
}
