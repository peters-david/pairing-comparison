use std::sync::{Condvar, Mutex};

pub struct Semaphore {
    max: usize,
    current: Mutex<usize>,
    condvar: Condvar,
}

impl Semaphore {
    pub fn new(max: usize) -> Self {
        Self {
            max,
            current: Mutex::new(0),
            condvar: Condvar::new(),
        }
    }

    pub fn acquire(&self) {
        let mut current = self.current.lock().expect("Could not lock current mutex");
        while *current >= self.max {
            current = self
                .condvar
                .wait(current)
                .expect("Could not wait for condvar");
        }
        *current += 1;
    }

    pub fn release(&self) {
        let mut current = self.current.lock().expect("Could not lock current mutex");
        *current -= 1;
        self.condvar.notify_one();
    }
}
