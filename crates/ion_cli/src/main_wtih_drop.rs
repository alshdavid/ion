use std::{collections::HashMap, ptr::NonNull};

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

    // Globals
    let mut isolate = v8::Isolate::new(v8::CreateParams::default());
    let isolate_ptr = isolate.as_mut() as *mut v8::Isolate;

    let context_ptr = {
        let mut scope = v8::HandleScope::new(&mut isolate);
        let context = v8::Context::new(&mut scope, Default::default());
        v8::Global::new(unsafe { &mut *isolate_ptr }, context).into_raw()
    };

    // Weak must outlive the scope
    let weak = {
        let scope = &mut v8_global_scope(isolate_ptr, context_ptr);

        // let value = v8::String::new(scope, "Hello World").unwrap();
        let value_ptr = Box::into_raw(Box::new(DropDetector{})) as _;
        let value = v8::External::new(scope, value_ptr);
        
        let weak = v8::Weak::with_guaranteed_finalizer(
            v8_isolate_ptr(isolate_ptr),
            value,
            Box::new(move || {
                println!("Called: with_guaranteed_finalizer");
                unsafe { Box::from_raw(value_ptr as *mut DropDetector); };
            }),
        );

        weak
        // weak.to_local(scope).unwrap()

        

    };

    {
        let scope = &mut v8_global_scope(isolate_ptr, context_ptr);
        v8_eval_code(scope, CODE);
    };

    isolate.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
    Ok(())
}

pub fn v8_eval_code<'a, S: AsRef<str>>(
    scope: &mut v8::HandleScope<'a>,
    code: S,
) -> v8::Global<v8::Value> {
    let code = v8::String::new(scope, code.as_ref()).unwrap();
    let script = v8::Script::compile(scope, code, None).unwrap();
    let value = script.run(scope).unwrap();
    v8::Global::new(scope, value)
}

pub fn v8_global_context(context_ptr: NonNull<v8::Context>) -> v8::Local<'static, v8::Context> {
    // SAFETY: `v8::Local` is always non-null pointer; the `HandleScope` is
    // already on the stack, but we don't have access to it.
    unsafe { std::mem::transmute::<NonNull<v8::Context>, v8::Local<v8::Context>>(context_ptr) }
}

pub fn v8_global_scope(
    isolate_ptr: *mut v8::Isolate,
    context_ptr: NonNull<v8::Context>,
) -> v8::HandleScope<'static> {
    v8::HandleScope::with_context(v8_isolate_ptr(isolate_ptr), v8_global_context(context_ptr))
}

pub fn v8_isolate_ptr(isolate_ptr: *mut v8::Isolate) -> &'static mut v8::Isolate {
    unsafe { &mut *isolate_ptr }
}


struct DropDetector {}

impl DropDetector {
    fn hello(&self) {
        println!("Hello")
    }
}

impl Drop for DropDetector {
    fn drop(&mut self) {
        println!("Dropped")
    }
}
