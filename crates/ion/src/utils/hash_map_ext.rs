use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::hash::Hash;

pub trait HashMapExt<K, V, S> {
    fn try_get_mut<Q>(
        &mut self,
        k: &Q,
    ) -> crate::Result<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized;

    fn try_get<Q>(
        &self,
        k: &Q,
    ) -> crate::Result<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized;

    fn try_remove<Q>(
        &mut self,
        k: &Q,
    ) -> crate::Result<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized;
}

impl<K, V, S> HashMapExt<K, V, S> for HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    fn try_get_mut<Q>(
        &mut self,
        k: &Q,
    ) -> crate::Result<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let Some(value) = self.get_mut(k) else {
            return Err(crate::Error::ValueGetError);
        };
        Ok(value)
    }

    fn try_get<Q>(
        &self,
        k: &Q,
    ) -> crate::Result<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let Some(value) = self.get(k) else {
            return Err(crate::Error::ValueGetError);
        };
        Ok(value)
    }

    fn try_remove<Q>(
        &mut self,
        k: &Q,
    ) -> crate::Result<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let Some(value) = self.remove(k) else {
            return Err(crate::Error::ValueGetError);
        };
        Ok(value)
    }
}
