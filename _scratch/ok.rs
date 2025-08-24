mod napi;

static CODE: &str = r#"
  "Hello World"
"#;

fn main() -> anyhow::Result<()> {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    let mut isolate = v8::Isolate::new(v8::CreateParams::default());
    let mut handle_scope = v8::HandleScope::new(&mut isolate);
    let context = v8::Context::new(&mut handle_scope, Default::default());
    let mut context_scope = v8::ContextScope::new(&mut handle_scope, context);

    let env = napi::Env::new(isolate_ptr, context, global, buffer_constructor, report_error)

    let code = v8::String::new(&mut context_scope, CODE).unwrap();

    let script = v8::Script::compile(&mut context_scope, code, None).unwrap();
    let result = script.run(&mut context_scope).unwrap();

    let result = result.to_string(&mut context_scope).unwrap();
    println!("{}", result.to_rust_string_lossy(&mut context_scope));
    Ok(())
}

fn v8_create_function_from_closure() {}
