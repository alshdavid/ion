use std::cell::RefCell;
use std::ffi::c_void;
use std::path::Path;
use std::path::PathBuf;

use crate::JsValue;
use crate::ResolverContext;
use crate::TransformerContext;
use crate::platform::JsRealm;
use crate::platform::resolve::run_resolvers;
use crate::utils::PathExt;
use crate::utils::v8::v8_create_string;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum ModuleStatus {
    Ready,
    Initializing,
    Uninitialized,
}

#[derive(Debug)]
pub struct Module {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) status: RefCell<ModuleStatus>,
    module: *mut c_void,
}

impl Module {
    pub fn new(
        realm: &JsRealm,
        name: impl AsRef<str>,
        source: impl AsRef<str>,
    ) -> crate::Result<Self> {
        let env = realm.env();

        let scope = &mut env.scope();

        let v8_name = v8_create_string(scope, name.as_ref())?;
        let v8_source = v8_create_string(scope, source.as_ref())?;

        let origin = v8::ScriptOrigin::new(
            scope,
            v8_name.into(),
            0,
            0,
            false,
            0,
            None,
            false,
            false,
            true,
            None,
        );

        let mut source = v8::script_compiler::Source::new(v8_source, Some(&origin));

        let Some(module) = v8::script_compiler::compile_module(scope, &mut source) else {
            return Err(crate::Error::ValueCreateError);
        };
        let id: i32 = module.get_identity_hash().into();

        // SAFETY: Dropped in Module::drop
        let module_ptr = Box::into_raw(Box::new(module));

        Ok(Self {
            id,
            module: module_ptr as _,
            name: name.as_ref().to_string(),
            status: RefCell::new(ModuleStatus::Initializing),
        })
    }

    pub fn update_status(
        &self,
        status: ModuleStatus,
    ) {
        let mut st = self.status.borrow_mut();
        (*st) = status
    }

    pub fn get_status(&self) -> ModuleStatus {
        let st = self.status.borrow();
        st.clone()
    }

    pub fn v8_module(&self) -> v8::Local<'static, v8::Module> {
        unsafe { *(self.module as *mut v8::Local<'static, v8::Module>) }
    }

    pub fn v8_initialize(
        is_entry: bool,
        realm: &JsRealm,
        name: impl AsRef<str>,
        referrer_path: impl AsRef<Path>,
    ) -> crate::Result<v8::Local<'static, v8::Module>> {
        let module_map = realm.module_map();

        if let Some(module) = module_map.get_module(&name) {
            if module.get_status() == ModuleStatus::Initializing {
                return Ok(module.v8_module());
            }
            if module.get_status() == ModuleStatus::Ready {
                return Ok(module.v8_module());
            }
        };

        let Some(result) = realm
            .background_blocking({
                let ctx = ResolverContext {
                    fs: realm.fs.clone(),
                    specifier: name.as_ref().to_string(),
                    from: PathBuf::from(referrer_path.as_ref()),
                };
                let resolvers = realm.resolvers.clone();
                async move { run_resolvers(&resolvers, ctx).await }
            })
            .unwrap()
        else {
            return Err(crate::Error::FileNotFound(name.as_ref().to_string()));
        };

        let code = match result.kind.as_str() {
            "js" => String::from_utf8(result.code)?,
            ext => {
                let Some(transformer) = realm.transformers.get(ext) else {
                    return Err(crate::Error::NoTransformerError(name.as_ref().to_string()));
                };

                let transform_result = (*transformer.transformer)(TransformerContext {
                    kind: ext.to_string(),
                    path: result.path.clone(),
                    content: result.code,
                })?;

                transform_result.code
            }
        };

        let module = Module::new(realm, result.path.try_to_string()?, code)?;

        let v8_module = module.v8_module();
        module_map.insert(module);

        let env = realm.env();
        let scope = &mut env.scope();

        if is_entry {
            scope.set_host_initialize_import_meta_object_callback(init_meta_callback);

            v8_module
                .instantiate_module(scope, Module::v8_initialize_callback)
                .unwrap();

            let promise = v8_module.evaluate(scope).unwrap().cast::<v8::Promise>();
            scope.perform_microtask_checkpoint();
            promise.result(scope);
        }

        let module = module_map
            .get_module(result.path.try_to_string().unwrap())
            .unwrap();
        module.update_status(ModuleStatus::Ready);

        if v8_module.get_status() == v8::ModuleStatus::Errored {
            let key = v8::String::new(scope, "message").unwrap().into();
            println!(
                "Error: {:?}",
                v8_module
                    .get_exception()
                    .cast::<v8::Object>()
                    .get(scope, key)
                    .unwrap()
                    .to_rust_string_lossy(scope)
            );
        }

        Ok(v8_module)
    }

    pub fn v8_run_module(
        is_entry: bool,
        realm: &JsRealm,
        module_name: String,
        module: Module,
    ) -> crate::Result<v8::Local<'static, v8::Module>> {
        let module_map = realm.module_map();

        let v8_module = module.v8_module();
        module_map.insert(module);

        let env = realm.env();
        let scope = &mut env.scope();

        if is_entry {
            scope.set_host_initialize_import_meta_object_callback(init_meta_callback);

            v8_module
                .instantiate_module(scope, Module::v8_initialize_callback)
                .unwrap();

            let promise = v8_module.evaluate(scope).unwrap().cast::<v8::Promise>();
            scope.perform_microtask_checkpoint();
            promise.result(scope);
        }

        let module = module_map.get_module(&module_name).unwrap();

        module.update_status(ModuleStatus::Ready);

        if v8_module.get_status() == v8::ModuleStatus::Errored {
            let key = v8::String::new(scope, "message").unwrap().into();
            println!(
                "Error: {:?}",
                v8_module
                    .get_exception()
                    .cast::<v8::Object>()
                    .get(scope, key)
                    .unwrap()
                    .to_rust_string_lossy(scope)
            );
        }

        Ok(v8_module)
    }

    // Called by v8_initialize
    pub(crate) fn v8_initialize_callback<'a>(
        context: v8::Local<'a, v8::Context>,
        specifier: v8::Local<'a, v8::String>,
        _import_attributes: v8::Local<'a, v8::FixedArray>,
        referrer: v8::Local<'a, v8::Module>,
    ) -> Option<v8::Local<'a, v8::Module>> {
        let scope = &mut unsafe { v8::CallbackScope::new(context) };

        let realm = JsRealm::v8_revive(scope);
        let specifier = specifier.to_rust_string_lossy(scope);
        let referrer_module = realm
            .module_map()
            .get_module_by_id(&referrer.get_identity_hash().into())
            .expect("Could not find parent path");

        let v8_module =
            Self::v8_initialize(false, realm, specifier, &referrer_module.name).unwrap();

        Some(v8_module)
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        drop(unsafe { Box::from_raw(self.module as *mut v8::Local<'static, v8::Module>) })
    }
}

pub(crate) unsafe extern "C" fn init_meta_callback(
    context: v8::Local<v8::Context>,
    module: v8::Local<v8::Module>,
    meta: v8::Local<v8::Object>,
) {
    let scope = &mut unsafe { v8::CallbackScope::new(context) };
    let realm = JsRealm::v8_revive(scope);
    let env = realm.env();

    // Extensions
    // TEMP, use data or statics or something
    {
        let global_this = env.global_this().unwrap();
        let global_this = global_this.value().cast::<v8::Object>();
        let key = v8::Integer::new(scope, module.get_identity_hash().into());
        if let Some(exports) = global_this.get(scope, key.into()) {
            global_this.delete(scope, key.into()).unwrap();
            let key = v8::String::new(scope, "extension").unwrap();
            meta.create_data_property(scope, key.into(), exports);
        };
    };
}
