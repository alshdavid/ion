/*
    This is testing Ion's context switching behavior.

    v8 uses "scope" handles to track values for GC. A consumer
    must build "scope"s up in a stack starting from an root scope up
    to the scope used to manage the values themselves

    Isolate
    └─ IsolateScope
       └─ ContextScope
          └─ HandleScope
             ├─ JsObject
             └─ JsNumber

    When a "scope" is dropped, v8 looks at the values associated
    with that scope and determines if those values should be
    garbage collected.

    You can only have one scope on the stack at any given time and
    must manually drop the scopes on the stack in order of creation.

    If you create a scope while another scope is active, v8 will throw
    an error. This can happen when asynchronous tasks are spawned.

    Isolate
    └─ IsolateScope
       └─ ContextScope
          ├─ HandleScope // Scope0
          │  ├─ JsObject
          │  ├─ JsNumber
          └─ HandleScope // Error: "Scope0 already exists"
             └─ JsNumber

    Additionally, if there are multiple v8::Contexts sharing an Isolate
    and you want to switch between them (as we do in Ion), their scopes
    must be unwound/dropped correctly and new root scopes must be created

    Isolate
    ├─ IsolateScope (ctx0) // Scope0
    │  └─ ContextScope
    └─ IsolateScope (ctx1) // Error: "Scope0 already exists"
       └─ ContextScope

    To switch between multiple v8 Contexts correctly you must:

    Step 1: Create a root scope for that context

    Isolate
    └─ IsolateScope (ctx0)
       └─ ContextScope

    Step 2: Create children scopes to handle interacting with values

    Isolate
    └─ IsolateScope (ctx0)
       └─ ContextScope
          └─ HandleScope
             └─ JsObject

    Step 3: To switch to a new context, drop all scopes

    Isolate
    └─ None

    Step 4: Create a root scope for the second context

    Isolate
    └─ IsolateScope (ctx1)
       └─ ContextScope

    Step 5: Create children scopes to handle interacting with values

    Isolate
    └─ IsolateScope (ctx1)
       └─ ContextScope
          └─ HandleScope
             └─ JsObject

    Step 6: Rinse and repeat every time you need to switch between
    contexts to interact with the values held in that context
*/
use ion::*;

pub fn main() -> anyhow::Result<()> {
    let runtime = JsRuntime::initialize_debug()?;

    let worker = runtime.spawn_worker()?;

    println!("[ctx0] Started");
    let ctx0 = worker.create_context()?;

    println!("[ctx1] Started");
    let ctx1 = worker.create_context()?;

    println!("[ctx2] Started");
    let ctx2 = worker.create_context()?;

    ctx0.exec_blocking(|env| {
        let value = env.eval_script::<JsNumber>("1 + 1")?;
        let result = value.get_u32()?;
        println!("[ctx0]: {}", result);
        Ok(())
    })?;

    ctx1.exec_blocking(|env| {
        let value = env.eval_script::<JsNumber>("1 + 1")?;
        let result = value.get_u32()?;
        println!("[ctx1]: {}", result);
        Ok(())
    })?;

    // Make sure contexts dropped clean up properly
    drop(ctx0);

    ctx2.exec_blocking(|env| {
        let value = env.eval_script::<JsNumber>("1 + 1")?;
        let result = value.get_u32()?;
        println!("[ctx1]: {}", result);
        Ok(())
    })?;

    // For illustrative purposes
    drop(ctx1);
    drop(ctx2);

    Ok(())
}
