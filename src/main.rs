use std::clone;

mod js;

static CODE: &str = r#"
  "Hello World"
"#;

fn main() -> anyhow::Result<()> {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::set_flags_from_string(
        "--no_freeze_flags_after_init --expose_gc --harmony-shadow-realm --allow_natives_syntax --turbo_fast_api_calls --js-source-phase-imports",
      );
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    let mut isolate = v8::Isolate::new(v8::CreateParams::default());

    let mut main_scope = v8::HandleScope::new(&mut isolate);
    let context = v8::Context::new(&mut main_scope, Default::default());
    let mut context_scope = v8::ContextScope::new(&mut main_scope, context);
    let mut scope = v8::HandleScope::new(&mut context_scope);

    let weak = {
        let mut scope = v8::HandleScope::new(&mut scope);
        let target = v8::Object::new(&mut scope);
        let weak = v8::Weak::with_guaranteed_finalizer(
            &mut scope,
            target,
            Box::new(|| println!("Called: with_guaranteed_finalizer")),
        );

        let code = v8::String::new(&mut scope, CODE).unwrap();
        let script = v8::Script::compile(&mut scope, code, None).unwrap();
        let result = script.run(&mut scope).unwrap();

        let result = result.to_string(&mut scope).unwrap();
        println!("{}", result.to_rust_string_lossy(&mut scope));

        weak
    };

    scope.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);

    Ok(())
}

/*

use std::clone;

mod js;

static CODE: &str = r#"
  globalThis.foo("Hello World")
"#;

fn main() -> anyhow::Result<()> {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::set_flags_from_string(
        "--no_freeze_flags_after_init --expose_gc --harmony-shadow-realm --allow_natives_syntax --turbo_fast_api_calls --js-source-phase-imports",
      );
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    let mut isolate = v8::Isolate::new(v8::CreateParams::default());

    let mut handle_scope = v8::HandleScope::new(&mut isolate);
    let context = v8::Context::new(&mut handle_scope, Default::default());
    let mut context_scope = v8::ContextScope::new(&mut handle_scope, context);

    let global_this = context.global(&mut context_scope);
    let js_fn_foo = js::v8_create_function_from_closure(
        &mut context_scope,
        |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, rv: v8::ReturnValue| {
            println!("Called")
        },
    );
    js::v8_set_obj_property(&mut context_scope, &global_this, "foo", js_fn_foo.into());

    let code = v8::String::new(&mut context_scope, CODE).unwrap();
    let script = v8::Script::compile(&mut context_scope, code, None).unwrap();
    let result = script.run(&mut context_scope).unwrap();

    js::v8_delete_obj_property(&mut context_scope, &global_this, "foo");
    let result = result.to_string(&mut context_scope).unwrap();
    println!("{}", result.to_rust_string_lossy(&mut context_scope));

    context_scope.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);

    Ok(())
}




*/
