use std::cell::RefCell;
use std::rc::Rc;

/// Simple single threaded reference counter
#[derive(Debug, Clone)]
pub struct RefCounter(Rc<RefCell<usize>>);

impl Default for RefCounter {
    fn default() -> Self {
        Self::new(1)
    }
}

impl RefCounter {
    pub fn new(start: usize) -> Self {
        Self(Rc::new(RefCell::new(start)))
    }

    pub fn inc(&self) -> usize {
        let mut count = self.0.borrow_mut();
        (*count) += 1;
        *count
    }

    pub fn dec(&self) -> bool {
        let mut count = self.0.borrow_mut();
        if *count == 0 {
            panic!("Cannot decrement below 0")
        }
        (*count) -= 1;
        *count == 0
    }

    pub fn count(&self) -> usize {
        *self.0.borrow()
    }
}
