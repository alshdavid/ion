use std::rc::Rc;

use crate::js::{
    v8_create_function_from_closure, v8_eval_code, v8_get_obj_property, v8_global_context,
    v8_global_scope,
};

mod js;

static CODE: &str = r#"
    globalThis.handler = (value) => {
        value()
        return "Hello World"
    }
"#;

fn main() -> anyhow::Result<()> {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::set_flags_from_string(
        "--no_freeze_flags_after_init --expose_gc --harmony-shadow-realm --allow_natives_syntax --turbo_fast_api_calls --js-source-phase-imports",
    );
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    let mut isolate = v8::Isolate::new(v8::CreateParams::default());
    let mut isolate_ptr = isolate.as_mut() as *mut v8::Isolate;

    let context_ptr = {
        let mut scope = v8::HandleScope::new(&mut isolate);
        let context = v8::Context::new(&mut scope, Default::default());
        v8::Global::new(unsafe { &mut *isolate_ptr }, context).into_raw()
    };

    let drop_detector = Rc::new(DropDetector {});
    let value = v8_create_function_from_closure(isolate_ptr, context_ptr, {
        let drop_detector = Rc::clone(&drop_detector);
        move |_, _, _| {
            drop_detector.hello();
        }
    });

    {
        let context = &mut v8_global_context(context_ptr);
        let scope = &mut v8_global_scope(isolate_ptr, context_ptr);

        v8_eval_code(scope, CODE);

        let global_this = context.global(scope);
        let callback = v8_get_obj_property(scope, &global_this, "handler")
            .unwrap()
            .try_cast::<v8::Function>()
            .unwrap();

        let value_l = v8::Local::new(scope, value);
        let mut recv = v8::undefined(scope);
        callback.call(scope, recv.into(), &[value_l.into()]);
    }

    isolate.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
    Ok(())
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


  let value = Box::new(DropDetector {});
        let value_ref = Box::into_raw(value);
        let external = v8::External::new(scope, value_ref as _);

        let callback = v8::Function::builder(
            move |scope: &mut v8::HandleScope,
                  args: v8::FunctionCallbackArguments,
                  rv: v8::ReturnValue| {
                println!("Running callback");
            },
        )
        .data(external.into())
        .build(scope)
        .unwrap();
