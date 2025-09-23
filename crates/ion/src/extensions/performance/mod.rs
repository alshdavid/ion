use tokio::time::Instant;

use crate::Env;
use crate::JsExtension;
use crate::JsFunction;
use crate::JsNumber;
use crate::JsObject;
use crate::JsObjectValue;

static MODULE_NAME: &str = "ion:performance";
static BINDING: &str = include_str!("./binding.js");

fn extension_hook(
    env: &Env,
    exports: &mut JsObject,
) -> crate::Result<()> {
    exports.set_named_property(
        "now",
        JsFunction::new(env, |env, _| {
            let now = Instant::now();
            // Convert the internal nanoseconds to milliseconds
            let elapsed = now.elapsed().as_nanos() as f64 / 1_000_000.0;

            JsNumber::from_f64(env, elapsed)
        })?,
    )?;

    Ok(())
}

pub fn performance() -> JsExtension {
    JsExtension::NativeModuleWithBinding {
        module_name: MODULE_NAME.to_string(),
        binding: BINDING.to_string(),
        extension: Box::new(extension_hook),
    }
}
