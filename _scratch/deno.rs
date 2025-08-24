mod napi;

static CODE: &str = r#"
  "Hello World"
"#;

fn main() -> anyhow::Result<()> {
    // deno_core::v8::Platform::new(0, false)
    deno_core::JsRuntime::init_platform(None, false);

    let mut runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: None,
        extension_code_cache: None,
        extension_transpiler: None,
        op_metrics_factory_fn: None,
        extensions: vec![],
        startup_snapshot: None,
        skip_op_registration: false,
        create_params: None,
        v8_platform: None,
        shared_array_buffer_store: None,
        compiled_wasm_module_store: None,
        inspector: false,
        is_main: false,
        validate_import_attributes_cb: None,
        wait_for_inspector_disconnect_callback: None,
        custom_module_evaluation_cb: None,
        eval_context_code_cache_cbs: None,
        import_assertions_support: deno_core::ImportAssertionsSupport::Error,
        maybe_op_stack_trace_callback: None,
    });

    runtime.execute_script("Something", "globalThis.foo = 42")?;

    let isolate = runtime.v8_isolate();
    let mut handle_scope = deno_core::v8::HandleScope::new(isolate);
    let context = deno_core::v8::Context::new(&mut handle_scope, Default::default());
    let mut context_scope = deno_core::v8::ContextScope::new(&mut handle_scope, context);

    let global_this = context.global(&mut context_scope);
    let key = deno_core::v8::String::new(&mut context_scope, "foo").unwrap();
    let result = global_this.get(&mut context_scope, key.into()).unwrap();

    let result = result.to_string(&mut context_scope).unwrap();
    println!("{}", result.to_rust_string_lossy(&mut context_scope));

    Ok(())
}

fn v8_create_function_from_closure() {}
