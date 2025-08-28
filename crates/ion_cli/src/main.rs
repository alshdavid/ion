
use std::{str::FromStr, sync::Arc};

use http::{HeaderName, HeaderValue, StatusCode};
use tokio::io::{AsyncWriteExt, DuplexStream};

use crate::http1::ResponseBuilderExt;
use crate::jsrt::{ContextPool, ContextPoolOptions, Environment};

mod http1;
mod js;
mod jsrt;
mod napi;

const SCRIPT: &str = include_str!("../handlers/hello-world.js");

fn main() -> anyhow::Result<()> {
    let pool = Arc::new(ContextPool::new(&ContextPoolOptions { threads: 8 }));
  
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(num_cpus::get_physical())
        .build()
        .unwrap()
        .block_on(async move {
            http1::http1_server("0.0.0.0:8080", move |_req, res| {
                let pool = pool.clone();
                async move {
                
                    pool.exec(move |mut env| {
                        // Evaluate FaaS Function
                        eval_script(&mut env, "globalThis.handler = () => 42");

                        // Get the exported handler
                        let js_fn_handler = get_handler(&mut env);

                        // Execute Js Handler
                        execute_handler(&mut js_fn_handler);

                        // Call FaaS handler
                        let js_fn_result = v8::undefined(&mut env.context_scope);
                        js_fn_handler.call(
                            &mut env.context_scope,
                            js_fn_result.into(),
                            &[],
                        );

                        println!("finished handler")
                    });

                    Ok(res.status(200).body(r#"Done"#))
                }
            })
            .await
        })
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

