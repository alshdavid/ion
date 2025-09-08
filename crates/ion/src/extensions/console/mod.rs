use crate::Env;
use crate::JsExtension;
use crate::JsFunction;
use crate::JsObject;
use crate::JsObjectValue;
use crate::JsString;
use crate::JsUnknown;

static MODULE_NAME: &str = "ion:console";
static BINDING: &str = include_str!("./binding.js");

fn extension_hook(
    env: &Env,
    exports: &mut JsObject,
) -> crate::Result<()> {
    exports.set_named_property(
        "log",
        JsFunction::new(env, |env, ctx| {
            let global_this = env.global_this()?;

            let json = global_this.get_named_property_unchecked::<JsObject>("JSON")?;
            let json_stringify = json.get_named_property_unchecked::<JsFunction>("stringify")?;

            let mut args_string = vec![];

            for i in 0..ctx.len() {
                let arg = ctx.arg::<JsUnknown>(i)?;
                let replacer = env.get_null()?;
                let spaces = env.create_int32(2)?;
                let result: JsString = json_stringify.call_with_args((arg, replacer, spaces))?;
                args_string.push((result.get_string()?).to_string());
            }

            let output = args_string.join(", ");
            println!("{}", output);

            Ok(())
        })?,
    )?;

    exports.set_named_property(
        "warn",
        JsFunction::new(env, |env, ctx| {
            let global_this = env.global_this()?;

            let json = global_this.get_named_property_unchecked::<JsObject>("JSON")?;
            let json_stringify = json.get_named_property_unchecked::<JsFunction>("stringify")?;

            let mut args_string = vec![];

            for i in 0..ctx.len() {
                let arg = ctx.arg::<JsUnknown>(i)?;
                let replacer = env.get_null()?;
                let spaces = env.create_int32(2)?;
                let result: JsString = json_stringify.call_with_args((arg, replacer, spaces))?;
                args_string.push((result.get_string()?).to_string());
            }

            let output = args_string.join(", ");
            println!("{}", output);

            Ok(())
        })?,
    )?;

    exports.set_named_property(
        "error",
        JsFunction::new(env, |env, ctx| {
            let global_this = env.global_this()?;

            let json = global_this.get_named_property_unchecked::<JsObject>("JSON")?;
            let json_stringify = json.get_named_property_unchecked::<JsFunction>("stringify")?;

            let mut args_string = vec![];

            for i in 0..ctx.len() {
                let arg = ctx.arg::<JsUnknown>(i)?;
                let replacer = env.get_null()?;
                let spaces = env.create_int32(2)?;
                let result: JsString = json_stringify.call_with_args((arg, replacer, spaces))?;
                args_string.push((result.get_string()?).to_string());
            }

            let output = args_string.join(", ");
            eprintln!("{}", output);

            Ok(())
        })?,
    )?;

    Ok(())
}

pub fn console() -> JsExtension {
    JsExtension::NativeModuleWithBinding {
        module_name: MODULE_NAME.to_string(),
        binding: BINDING.to_string(),
        extension: Box::new(extension_hook),
    }
}
