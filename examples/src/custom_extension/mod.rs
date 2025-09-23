use ion::*;

pub fn main() -> anyhow::Result<()> {
    // Start the runtime
    let runtime = JsRuntime::initialize_once(JsRuntimeOptions {
        extensions: vec![custom_extension()],
        ..Default::default()
    })?;

    let worker = runtime.spawn_worker()?;
    let ctx = worker.create_context()?;

    ctx.exec_blocking(|env| {
        let module = env.eval_module(
            r#"
            import { foo } from "ion:foo";

            export default foo()
        "#,
        )?;

        let default_export = module.get_named_property_unchecked::<JsString>("default")?;
        println!("Got: {}", default_export.get_string()?);
        Ok(())
    })?;

    Ok(())
}

fn custom_extension() -> JsExtension {
    JsExtension::NativeModuleWithBinding {
        module_name: "ion:foo".to_string(),
        binding: r#"
            export function foo() {
                return import.meta.extension.foo
            }
        "#
        .to_string(),
        extension: Box::new(|env, exports| {
            let key = env.create_string("foo")?;
            let value = env.create_string("bar")?;
            exports.set_property(key, value)?;
            Ok(())
        }),
    }
}
