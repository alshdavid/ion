use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::platform::module::Module;

#[derive(Debug, Default, Clone)]
pub struct ModuleMap {
    inner: Rc<RefCell<HashMap<i32, Rc<Module>>>>,
    names: Rc<RefCell<HashMap<String, i32>>>,
}

impl ModuleMap {
    pub fn insert(
        &self,
        module: Module,
    ) {
        let id = module.id;
        let name = module.name.clone();

        let mut inner = self.inner.borrow_mut();
        let mut names = self.names.borrow_mut();

        inner.insert(id, Rc::new(module));
        names.insert(name, id);
    }

    pub fn get_module_by_id(
        &self,
        id: &i32,
    ) -> Option<Rc<Module>> {
        let inner = self.inner.borrow();
        if let Some(module) = inner.get(id) {
            return Some(Rc::clone(&module));
        }
        None
    }

    pub fn get_module(
        &self,
        name: impl AsRef<str>,
    ) -> Option<Rc<Module>> {
        let names = self.names.borrow();
        let id = names.get(name.as_ref())?;
        self.get_module_by_id(id)
    }
}
