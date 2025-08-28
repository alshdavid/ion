pub struct MemoryStats {
    pub allocated: Bytes,
    pub resident: Bytes,
}

impl MemoryStats {
    pub  fn current() -> MemoryStats {
        jemalloc_ctl::epoch().unwrap();
        MemoryStats {
            allocated: Bytes(jemalloc_ctl::stats::allocated().unwrap()),
            resident: Bytes(jemalloc_ctl::stats::resident().unwrap()),
        }
    }
}

impl std::fmt::Display for MemoryStats {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "{} allocated {} resident",
            self.allocated, self.resident,
        )
    }
}

#[derive(Default)]
pub struct Bytes(pub usize);

impl std::fmt::Display for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let bytes = self.0;
        if bytes < 4096 {
            return write!(f, "{} bytes", bytes);
        }
        let kb = bytes / 1024;
        if kb < 4096 {
            return write!(f, "{}kb", kb);
        }
        let mb = kb / 1024;
        write!(f, "{}mb", mb)
    }
}

impl std::ops::AddAssign<usize> for Bytes {
    fn add_assign(&mut self, x: usize) {
        self.0 += x;
    }
}