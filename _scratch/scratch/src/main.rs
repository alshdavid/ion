fn main() {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    let isolate = &mut v8::Isolate::new(v8::CreateParams::default());
    let handle_scope = &mut v8::HandleScope::new(isolate);

    let context = v8::Context::new(handle_scope, Default::default());
    let scope = &mut v8::ContextScope::new(handle_scope, context);

    let global = context.global(scope);

    //This js file just returns a promise wrapped "helloworld"
    let code = v8::String::new(scope, "globalThis.main = async () => 'Hello World'").unwrap();

    let script = v8::Script::compile(scope, code, None).unwrap();
    let result = script.run(scope).unwrap();

    let function_name_string =
        v8::String::new(scope, "main").expect("failed to convert Rust string to javascript string");

    let function = global
        .get(scope, function_name_string.into())
        .expect(&*format!("could not find function {}", "main"));
    let function: v8::Local<v8::Function> = v8::Local::cast(function);
    let recv = v8::Integer::new(scope, 1).into();
    let result = function.call(scope, recv, &[]).expect("couldnt run");

    println!("{}", result.is_promise());

    scope.perform_microtask_checkpoint();

    let promise = v8::Local::<v8::Promise>::try_from(result).unwrap();
    let promise_result = promise.result(scope);

    println!("{}", promise_result.to_rust_string_lossy(scope));
}
