use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use flume::Sender;

use crate::DynResolver;
use crate::Env;
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
    pub(crate) resolvers: Vec<DynResolver>,
    pub(crate) fs: FileSystem,
    pub(crate) id: usize,
    pub(crate) env: Box<Env>,
    pub(crate) background_task_manager: Arc<BackgroundTaskManager>,
    pub(crate) finalizer_registry: FinalizerRegistery,
    /// Used to tell the Worker if there are any long-lived async tasks
    /// that should prevent the context from being shutdown
    pub(crate) global_refs: RefCounter,
    pub(crate) shutdown_requested: Rc<RefCell<bool>>,
    pub(crate) modules: ModuleMap,
    pub(crate) tx: Sender<JsWorkerEvent>,
    pub(crate) global_this: sys::__v8_global_this,
    pub(crate) context: sys::__v8_context,
}

impl JsRealm {
    pub(crate) fn new(
        isolate: *mut v8::Isolate,
        fs: FileSystem,
        resolvers: Vec<DynResolver>,
        background_task_manager: Arc<BackgroundTaskManager>,
        tx: Sender<JsWorkerEvent>,
    ) -> Box<Self> {
        let context = {
            let handle_scope =
                sys::v8_new_root_scope(v8::HandleScope::new(unsafe { &mut *isolate }));
            let context = sys::v8_new_context(isolate, sys::v8_get_root_scope(handle_scope));
            sys::v8_drop_root_scope(handle_scope);
            context
        };

        let global_this = {
            let handle_scope =
                sys::v8_new_root_scope(v8::HandleScope::new(unsafe { &mut *isolate }));
            let context_scope = sys::v8_new_context_scope(v8::ContextScope::new(
                sys::v8_get_root_scope(handle_scope),
                sys::v8_get_context(context),
            ));
            let global_this = sys::v8_new_global_this(context, context_scope);
            sys::v8_drop_context_scope(context_scope);
            sys::v8_drop_root_scope(handle_scope);
            global_this
        };

        let global_refs = RefCounter::new(0);
        let shutdown_requested = Rc::new(RefCell::new(false));
        let finalizer_registry = FinalizerRegistery::new(isolate);

        // TODO make these RefCells
        let modules: ModuleMap = ModuleMap::default();

        let env = Env::new(
            isolate,
            context,
            global_this,
            Arc::clone(&background_task_manager),
            global_refs.clone(),
            Rc::clone(&shutdown_requested),
            tx.clone(),
            finalizer_registry.clone(),
        );

        let mut realm = Box::new(JsRealm {
            id: 0,
            env,
            fs,
            background_task_manager,
            modules,
            resolvers,
            global_refs,
            shutdown_requested,
            finalizer_registry,
            // v8 internals
            global_this,
            context,
            tx,
        });

        let realm_ptr = realm.as_mut() as *mut JsRealm;
        let realm_id = realm_ptr as usize;
        realm.env.realm_id = realm_id.clone();

        {
            // TODO use slot or data
            let handle_scope =
                sys::v8_new_root_scope(v8::HandleScope::new(unsafe { &mut *isolate }));
            let context_scope = sys::v8_new_context_scope(v8::ContextScope::new(
                sys::v8_get_root_scope(handle_scope),
                sys::v8_get_context(context),
            ));
            let scope = sys::v8_get_context_scope(context_scope);
            let key = v8::String::new(scope, "__data").unwrap();
            let value = v8::External::new(scope, realm_ptr as _);
            let global_this = sys::v8_get_context(realm.context).global(scope);
            global_this.set(scope, key.into(), value.into());

            sys::v8_drop_context_scope(context_scope);
            sys::v8_drop_root_scope(handle_scope);
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

    pub fn spawn_background(
        &self,
        fut: impl 'static + Send + Sync + Future<Output = crate::Result<()>>,
    ) -> crate::Result<()> {
        let tx = self.tx.clone();
        let id = self.id.clone();
        self.background_task_manager.spawn(async move {
            if let Err(_error) = fut.await {
                todo!("Missing global error handler")
            };
            Ok(tx.try_send(JsWorkerEvent::BackgroundTaskComplete { id })?)
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

    pub(crate) fn v8_revive<'a>(scope: &mut v8::HandleScope<'_>) -> &'a mut JsRealm {
        let context = scope.get_current_context();
        let global_this = context.global(scope);
        let data_key = v8::String::new(scope, "__data").unwrap();
        let data = global_this.get(scope, data_key.into()).unwrap();
        let data = data.cast::<v8::External>();
        unsafe { &mut *(data.value() as *mut JsRealm) }
    }
}
