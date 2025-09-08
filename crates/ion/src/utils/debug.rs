// Helpful for debugging, not needed in the application
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::LazyLock;
use std::sync::atomic::AtomicU64;

static ID: LazyLock<AtomicU64> = LazyLock::new(Default::default); //Default::default();

pub fn new_id() -> u64 {
    ID.fetch_add(1, std::sync::atomic::Ordering::AcqRel)
}

pub struct DropDetector<T>(u64, String, T);

impl<T> DropDetector<T> {
    pub fn new(
        name: impl AsRef<str>,
        v: T,
    ) -> Self {
        let id = new_id();
        println!("-> [{}] [{}] Created", id, name.as_ref());
        Self(id, name.as_ref().to_string(), v)
    }
}

impl<T> Drop for DropDetector<T> {
    fn drop(&mut self) {
        println!("<- [{}] [{}] Dropped", self.0, self.1);
    }
}

impl<T> Deref for DropDetector<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.2
    }
}

impl<T> DerefMut for DropDetector<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.2
    }
}
