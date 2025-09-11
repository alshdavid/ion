use std::collections::HashSet;
use std::sync::Arc;

use parking_lot::Mutex;

use crate::Env;
use crate::JsExtension;
use crate::JsFunction;
use crate::JsNumber;
use crate::JsObject;
use crate::JsObjectValue;
use crate::JsString;
use crate::ThreadSafeFunction;
use crate::thread_safe_function;
use crate::utils::AtomicRefCounter;

static MODULE_NAME: &str = "ion:timers/timeout";
static BINDING: &str = include_str!("./binding.js");

fn extension_hook(
    env: &Env,
    exports: &mut JsObject,
) -> crate::Result<()> {
    let timer_refs = Arc::new(Mutex::new(HashSet::<String>::new()));
    let ref_count = AtomicRefCounter::new(0);

    exports.set_named_property(
        "setTimeout",
        JsFunction::new(env, {
            let timer_refs = Arc::clone(&timer_refs);
            let ref_count = ref_count.clone();

            move |env, ctx| {
                let arg0 = ctx.arg::<JsFunction>(0)?;
                let arg1 = ctx.arg::<JsNumber>(1)?;

                let callback = ThreadSafeFunction::new(&arg0)?;
                let duration = arg1.get_u32()?;
                let timer_ref = format!("{}", ref_count.inc());

                {
                    let mut timer_refs = timer_refs.lock();
                    timer_refs.insert(timer_ref.clone());
                }

                env.spawn_background({
                    let timer_ref = timer_ref.clone();
                    let timer_refs = Arc::clone(&timer_refs);
                    async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(duration as u64))
                            .await;

                        {
                            let mut timer_refs = timer_refs.lock();
                            if !timer_refs.remove(&timer_ref) {
                                return Ok(());
                            }
                        }

                        callback
                            .call_async(
                                thread_safe_function::map_arguments::noop,
                                thread_safe_function::map_return::noop,
                            )
                            .await?;

                        Ok(())
                    }
                })?;

                Ok(timer_ref)
            }
        })?,
    )?;

    exports.set_named_property(
        "clearTimeout",
        JsFunction::new(env, {
            move |_env, ctx| {
                let arg0 = ctx.arg::<JsString>(0)?;

                {
                    let mut timer_refs = timer_refs.lock();
                    timer_refs.remove(&arg0.get_string()?);
                }

                Ok(())
            }
        })?,
    )?;

    Ok(())
}

pub fn set_timeout() -> JsExtension {
    JsExtension::NativeModuleWithBinding {
        module_name: MODULE_NAME.to_string(),
        binding: BINDING.to_string(),
        extension: Box::new(extension_hook),
    }
}
