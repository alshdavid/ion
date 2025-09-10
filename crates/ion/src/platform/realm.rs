use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use flume::Sender;

use crate::DynResolver;
use crate::Env;
use crate::fs::FileSystem;
use crate::platform::background_worker::BackgroundTaskManager;
use crate::platform::module_map::ModuleMap;
use crate::platform::v8::__v8_context;
use crate::platform::v8::__v8_global_this;
use crate::platform::v8::v8_drop_context_scope;
use crate::platform::v8::v8_drop_root_scope;
use crate::platform::v8::v8_get_context;
use crate::platform::v8::v8_get_context_scope;
use crate::platform::v8::v8_get_root_scope;
use crate::platform::v8::v8_new_context;
use crate::platform::v8::v8_new_context_scope;
use crate::platform::v8::v8_new_global_this;
use crate::platform::v8::v8_new_root_scope;
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
    pub(crate) modules: ModuleMap,
    pub(crate) tx: Sender<JsWorkerEvent>,
    pub(crate) global_this: __v8_global_this,
    pub(crate) context: __v8_context,
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
            let handle_scope = v8_new_root_scope(v8::HandleScope::new(unsafe { &mut *isolate }));
            let context = v8_new_context(isolate, v8_get_root_scope(handle_scope));
            v8_drop_root_scope(handle_scope);
            context
        };

        let global_this = {
            let handle_scope = v8_new_root_scope(v8::HandleScope::new(unsafe { &mut *isolate }));
            let context_scope = v8_new_context_scope(v8::ContextScope::new(
                v8_get_root_scope(handle_scope),
                v8_get_context(context),
            ));
            let global_this = v8_new_global_this(context, context_scope);
            v8_drop_context_scope(context_scope);
            v8_drop_root_scope(handle_scope);
            global_this
        };

        let global_refs = RefCounter::new(0);
        let shutdown_requested = Rc::new(RefCell::new(false));

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
        );

        // let context = unsafe {
        //     println!("{}", Rc::strong_count(&context));
        //     let c = Rc::into_raw(context);
        //     let context =Rc::from_raw(c);
        //     Rc::decrement_strong_count(&c);
        //     Rc::decrement_strong_count(&global_this);
        //     println!("{}", Rc::strong_count(&env.context));
        //     context
        // };

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
            let handle_scope = v8_new_root_scope(v8::HandleScope::new(unsafe { &mut *isolate }));
            let context_scope = v8_new_context_scope(v8::ContextScope::new(
                v8_get_root_scope(handle_scope),
                v8_get_context(context),
            ));
            let scope = v8_get_context_scope(context_scope);
            let key = v8::String::new(scope, "__data").unwrap();
            let value = v8::External::new(scope, realm_ptr as _);
            let global_this = v8_get_context(realm.context).global(scope);
            global_this.set(scope, key.into(), value.into());

            v8_drop_context_scope(context_scope);
            v8_drop_root_scope(handle_scope);
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
