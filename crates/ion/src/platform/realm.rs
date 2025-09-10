use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use flume::Sender;

use crate::DynResolver;
use crate::Env;
use crate::fs::FileSystem;
use crate::platform::background_worker::BackgroundTaskManager;
use crate::platform::module_map::ModuleMap;
use crate::platform::v8::RawContext;
use crate::platform::v8::RawContextScope;
use crate::platform::v8::RawGlobal;
use crate::platform::v8::RawIsolate;
use crate::platform::v8::RawIsolateScope;
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
    /// Used to tell the Worker if there are any long-lived async tasks
    /// that should prevent the context from being shutdown
    pub(crate) global_refs: RefCounter,
    pub(crate) shutdown_requested: Rc<RefCell<bool>>,
    // TODO make these RefCells
    pub(crate) modules: *mut ModuleMap,
    #[allow(unused)]
    pub(crate) tx: Sender<JsWorkerEvent>,
    // SAFETY: Changing the order of these properties
    // will affect their drop order and break the isolate
    #[allow(unused)]
    pub(crate) global_this: Rc<RawGlobal>,
    #[allow(unused)]
    pub(crate) context: Rc<RawContext>,
}

impl JsRealm {
    pub(crate) fn new(
        isolate: Rc<RawIsolate>,
        fs: FileSystem,
        resolvers: Vec<DynResolver>,
        background_task_manager: Arc<BackgroundTaskManager>,
        tx: Sender<JsWorkerEvent>,
    ) -> Box<Self> {
        let handle_scope = RawIsolateScope::new(v8::HandleScope::new(isolate.as_mut()));

        let context = RawContext::new(isolate.as_ref(), handle_scope.as_mut());
        let context_scope = RawContextScope::new(v8::ContextScope::new(
            handle_scope.as_mut(),
            context.as_inner(),
        ));

        let global_this = RawGlobal::new(&context, &context_scope);
        let global_refs = RefCounter::new(0);
        let shutdown_requested = Rc::new(RefCell::new(false));

        // TODO make these RefCells
        let modules = Box::into_raw(Box::new(ModuleMap::default()));

        let env = Env::new(
            isolate,
            Rc::clone(&context),
            Rc::clone(&global_this),
            Arc::clone(&background_task_manager),
            global_refs.clone(),
            Rc::clone(&shutdown_requested),
            tx.clone(),
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
            let scope = &mut unsafe { v8::CallbackScope::new(realm.context.as_inner()) };
            let key = v8::String::new(scope, "__data").unwrap();
            let value = v8::External::new(scope, realm_ptr as _);
            let global_this = realm.context.global(scope);
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
    pub(crate) fn module_map(&self) -> &mut ModuleMap {
        unsafe { &mut *self.modules }
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

impl Drop for JsRealm {
    fn drop(&mut self) {
        drop(unsafe { Box::from_raw(self.modules) });
    }
}
