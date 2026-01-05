use std::collections::HashMap;
use std::sync::Arc;

use flume::Sender;

use crate::Env;
use crate::FromJsValue;
use crate::JsObject;
use crate::JsResolver;
use crate::JsTransformer;
use crate::fs::FileSystem;
use crate::platform::background_worker::BackgroundTaskManager;
use crate::platform::finalizer_registry::FinalizerRegistery;
use crate::platform::module_map::ModuleMap;
use crate::platform::sys;
use crate::platform::worker::JsWorkerEvent;
use crate::utils::RefCounter;
use crate::utils::channel::oneshot;

// Container that constructs a V8 context and preserves the internals until dropped
pub struct JsRealm {
    pub(crate) resolvers: Vec<JsResolver>,
    pub(crate) transformers: HashMap<String, Arc<JsTransformer>>,
    pub(crate) fs: FileSystem,
    pub(crate) id: usize,
    pub(crate) env: Box<Env>,
    pub(crate) background_task_manager: Arc<BackgroundTaskManager>,
    pub(crate) finalizer_registry: FinalizerRegistery,
    /// Used to tell the Worker if there are any long-lived async tasks
    /// that should prevent the context from being shutdown
    pub(crate) global_refs: RefCounter,
    pub(crate) modules: ModuleMap,
    pub(crate) global_this: sys::GlobalThis,
}

impl JsRealm {
    pub(crate) fn new(
        isolate: *mut v8::Isolate,
        fs: FileSystem,
        resolvers: Vec<JsResolver>,
        transformers: HashMap<String, Arc<JsTransformer>>,
        background_task_manager: Arc<BackgroundTaskManager>,
        tx: Sender<JsWorkerEvent>,
    ) -> Box<Self> {
        let context = sys::GlobalContext::new(unsafe { &mut *isolate });
        let global_this = sys::GlobalThis::new(&context);

        let global_refs = RefCounter::new(0);
        let finalizer_registry = FinalizerRegistery::new(isolate);

        // TODO make these RefCells
        let modules: ModuleMap = ModuleMap::default();

        let env = Env::new(
            isolate,
            context.clone(),
            Arc::clone(&background_task_manager),
            global_refs.clone(),
            tx.clone(),
            finalizer_registry.clone(),
            global_this.clone(),
        );

        let mut realm = Box::new(JsRealm {
            id: 0,
            env,
            fs,
            background_task_manager,
            modules,
            resolvers,
            transformers,
            global_refs,
            finalizer_registry,
            global_this,
        });

        let realm_ptr = realm.as_mut() as *mut JsRealm;
        let realm_id = realm_ptr as usize;
        realm.env.realm_id = realm_id;

        {
            // TODO use slot or data
            let scope = &mut context.scope();
            let global_this = context.as_local().global(scope);
            let key = v8::String::new(scope, "__data").unwrap();
            let value = v8::External::new(scope, realm_ptr as _);
            global_this.set(scope, key.into(), value.into());
        }

        realm.id = realm_id;

        realm
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn env(&self) -> &Env {
        &self.env
    }

    pub fn global_this(&self) -> crate::Result<JsObject> {
        let global_this = sys::Value::new(self.global_this.as_local().into());
        JsObject::from_js_value(&self.env, global_this)
    }

    pub fn spawn_background(
        &self,
        fut: impl 'static + Send + Sync + Future<Output = crate::Result<()>>,
    ) -> crate::Result<()> {
        self.background_task_manager.spawn(async move {
            if let Err(_error) = fut.await {
                todo!("Missing global error handler")
            };
            Ok(())
        })
    }

    pub fn background_blocking<Return: 'static + Send + Sync>(
        &self,
        fut: impl 'static + Send + Sync + Future<Output = crate::Result<Return>>,
    ) -> crate::Result<Return> {
        let (tx, rx) = oneshot();
        self.background_task_manager.spawn(async move {
            tx.try_send(fut.await).unwrap();
            Ok(())
        })?;
        rx.recv()?
    }

    #[allow(clippy::mut_from_ref)]
    pub(crate) fn module_map(&self) -> ModuleMap {
        self.modules.clone()
    }

    pub(crate) fn v8_revive<'a>(env: &Env) -> &'a mut JsRealm {
        let context = env.context.as_local();
        let scope = &mut env.context.scope();
        let global_this = context.global(scope);
        let data_key = v8::String::new(scope, "__data").unwrap();
        let data = global_this.get(scope, data_key.into()).unwrap();
        let data = data.cast::<v8::External>();
        unsafe { &mut *(data.value() as *mut JsRealm) }
    }
}
