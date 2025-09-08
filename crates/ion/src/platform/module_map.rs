use std::collections::HashMap;

use crate::platform::module::Module;

#[derive(Debug, Default)]
pub struct ModuleMap {
    inner: HashMap<i32, Module>,
    names: HashMap<String, i32>,
}

impl ModuleMap {
    pub fn insert(
        &mut self,
        module: Module,
    ) {
        let id = module.id;
        let name = module.name.clone();

        self.inner.insert(id, module);
        self.names.insert(name, id);
    }

    pub fn get_module_by_id(
        &self,
        id: &i32,
    ) -> Option<&Module> {
        self.inner.get(id)
    }

    pub fn get_module_by_id_mut(
        &mut self,
        id: &i32,
    ) -> Option<&mut Module> {
        self.inner.get_mut(id)
    }

    pub fn get_module(
        &self,
        name: impl AsRef<str>,
    ) -> Option<&Module> {
        let id = self.names.get(name.as_ref())?;
        self.get_module_by_id(id)
    }

    pub fn get_module_mut(
        &mut self,
        name: impl AsRef<str>,
    ) -> Option<&mut Module> {
        let id = self.names.get_mut(name.as_ref())?;
        self.inner.get_mut(id)
    }
}
