use std::sync::Arc;

use crate::{
    http1::ResponseBuilderExt,
    jsrt::{ContextPool, ContextPoolOptions, Environment},
};

mod http1;
mod js;
mod jsrt;

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
            pool.exec_async(move |mut env| {
                // Evaluate FaaS Function
                eval_script(&mut env, SCRIPT);

                // Get the exported handler
                let jsv_fn_handler = get_handler(&mut env);

                // Create Rust <-> JavaScript bindings
                let jsv_http_response = v8_create_http_response(&mut env.context_scope)
                    .try_cast::<v8::Value>()
                    .expect("Cast to value");

                // Call FaaS handler
                let jsv_ukn_return = v8::undefined(&mut env.context_scope);
                jsv_fn_handler.call(
                    &mut env.context_scope,
                    jsv_ukn_return.into(),
                    &[jsv_http_response, jsv_http_response],
                );
            }).await;

            Ok(res.status(200).body_from("Hello world")?)
        }
    })
    .await?;

    Ok(())
}

fn v8_create_http_response<'a>(
    scope: &mut v8::ContextScope<'a, v8::HandleScope>,
) -> v8::Local<'a, v8::Object> {
    let jsv_obj_response = v8::Object::new(scope);

    let jsv_fn_write = js::v8_create_function_from_closure(
        scope,
        move |_scope: &mut v8::HandleScope,
              _args: v8::FunctionCallbackArguments,
              _rv: v8::ReturnValue| { println!("Run on JavaScript thread") },
    );

    js::v8_set_obj_property(scope, &jsv_obj_response, "write", jsv_fn_write.into());

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
