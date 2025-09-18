#![deny(unused_crate_dependencies)]
mod basic;
mod custom_extension;
mod custom_resolver;
mod deferred;
mod eval;
mod http_server;
mod promise;
mod run;
mod set_interval;
mod set_timeout;
mod testing;
mod thread_safe_function;
mod thread_safe_promise;

fn main() -> anyhow::Result<()> {
    let example = std::env::args()
        .collect::<Vec<String>>()
        .get(1)
        .cloned()
        .unwrap_or("basic".to_string());

    match example.as_str() {
        "basic" => basic::main(),
        "custom_extension" => custom_extension::main(),
        "custom_resolver" => custom_resolver::main(),
        "deferred" => deferred::main(),
        "eval" => eval::main(),
        "http_server" => http_server::main(),
        "promise" => promise::main(),
        "run" => run::main(),
        "set_interval" => set_interval::main(),
        "set_timeout" => set_timeout::main(),
        "testing_background_tasks" => testing::background_tasks::main(),
        "testing_memory_usage_context" => testing::memory_usage_context::main(),
        "testing_memory_usage_module" => testing::memory_usage_module::main(),
        "testing_memory_usage_tsfn" => testing::memory_usage_tsfn::main(),
        "testing_memory_usage_value" => testing::memory_usage_value::main(),
        "testing_memory_usage_worker" => testing::memory_usage_worker::main(),
        "testing_multiple_contexts" => testing::multiple_contexts::main(),
        "testing_multiple_workers" => testing::multiple_workers::main(),
        "testing_transformers" => testing::transformers::main(),
        "testing_typescript" => testing::typescript::main(),
        "testing_wait" => testing::wait::main(),
        "thread_safe_function" => thread_safe_function::main(),
        "thread_safe_promise" => thread_safe_promise::main(),
        _ => Err(anyhow::anyhow!("No example for: \"{}\"", example)),
    }
}
