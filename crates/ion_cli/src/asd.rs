pub(crate) mod js;
pub(crate) mod utils;

use std::{collections::HashMap, ptr::NonNull, rc::Rc, time::Duration};

use crate::utils::{Bytes, MemoryStats, v8_eval_code};

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

static BYTES: &[u8] = include_bytes!("../../../Cargo.lock");

fn main() -> anyhow::Result<()> {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::set_flags_from_string(
        "--no_freeze_flags_after_init --expose_gc --harmony-shadow-realm --allow_natives_syntax --turbo_fast_api_calls --js-source-phase-imports",
    );
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    main2()?;
    println!("Next {})", MemoryStats::current());

    // let mut prev = MemoryStats::current();
    // for i in 0..100 {
    //     for _ in 0..10000 {
    //         main2()?;
    //     }
    //     let res = MemoryStats::current();
    //     let diff = res.resident.0 - prev.resident.0;
    //     prev = res;
    //     println!("Next {}, {} ({})", i, prev, Bytes(diff));
    //     // std::thread::sleep(Duration::from_secs(1));
    // }

    // std::thread::sleep(Duration::from_secs(60));
    Ok(())
}

static CODE: &str = r#"
    "Hello World"
"#;

fn main2() -> anyhow::Result<()> {
    let mut isolate = v8::Isolate::new(v8::CreateParams::default());

    {
        let handle_scope = &mut v8::HandleScope::new(&mut isolate);
        let context = v8::Context::new(handle_scope, Default::default());
        let context_scope = &mut v8::ContextScope::new(handle_scope, context);

        let global_raw = {
            let scope = &mut v8::HandleScope::new(context_scope);
            let local_str = v8::String::new(scope, "Hello world").unwrap();
            let global_str = v8::Global::new(scope, local_str);
            let global_raw = Box::into_raw(Box::new(global_str));
            global_raw
        };

        {
            let scope = &mut v8::HandleScope::new(context_scope);
            let global_handle = unsafe { *Box::from_raw(global_raw) };
            let handle = v8::Local::new(scope, global_handle);
            drop(handle) // previously allocated value is never GC'd
        }
    }

    isolate.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
    Ok(())
}
