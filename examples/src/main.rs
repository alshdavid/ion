// #![deny(unused_crate_dependencies)]
mod _utils;
mod background_tasks;
mod basic;
mod context_multiplexing;
mod custom_extension;
mod custom_resolver;
mod deferred;
mod eval;
mod external_value;
// mod http_server;
mod memory_usage_context;
mod memory_usage_external_value;
mod memory_usage_module;
mod memory_usage_tsfn;
mod memory_usage_value;
mod memory_usage_worker;
mod multiple_workers;
mod promise;
mod run;
mod set_interval;
mod set_timeout;
mod thread_safe_function;
mod thread_safe_promise;
mod transformers;
mod typescript;
mod basic_join;

fn main() -> anyhow::Result<()> {
    let example = std::env::args()
        .collect::<Vec<String>>()
        .get(1)
        .cloned()
        .unwrap_or("basic".to_string());

    match example.as_str() {
        "basic" => basic::main(),
        "basic_join" => basic_join::main(),
        "custom_extension" => custom_extension::main(),
        "custom_resolver" => custom_resolver::main(),
        "deferred" => deferred::main(),
        "eval" => eval::main(),
        // "http_server" => http_server::main(),
        "promise" => promise::main(),
        "run" => run::main(),
        "set_interval" => set_interval::main(),
        "set_timeout" => set_timeout::main(),
        "context_multiplexing" => context_multiplexing::main(),
        "thread_safe_function" => thread_safe_function::main(),
        "thread_safe_promise" => thread_safe_promise::main(),
        "background_tasks" => background_tasks::main(),
        "multiple_workers" => multiple_workers::main(),
        "typescript" => typescript::main(),
        "transformers" => transformers::main(),
        "memory_usage_context" => memory_usage_context::main(),
        "memory_usage_tsfn" => memory_usage_tsfn::main(),
        "memory_usage_worker" => memory_usage_worker::main(),
        "memory_usage_module" => memory_usage_module::main(),
        "memory_usage_value" => memory_usage_value::main(),
        "external_value" => external_value::main(),
        "memory_usage_external_value" => memory_usage_external_value::main(),
        _ => Err(anyhow::anyhow!("No example for: \"{}\"", example)),
    }
}
