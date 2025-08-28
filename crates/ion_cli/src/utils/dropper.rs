pub struct DropDetector {}

impl DropDetector {
    pub fn hello(&self) {
        // println!("Hello")
    }
}

impl Default for DropDetector {
    fn default() -> Self {
        println!("Constructed");
        Self {  }
    }
}

impl Drop for DropDetector {
    fn drop(&mut self) {
        println!("Dropped")
    }
}

