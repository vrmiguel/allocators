use spin::{Mutex, MutexGuard};

/// A spinlock-based mutex
// TODO: consider other mutex implementations
pub struct Mutexed<T> {
    inner: Mutex<T>,
}

impl<T> Mutexed<T> {
    pub const fn new(inner: T) -> Self {
        Mutexed {
            inner: Mutex::new(inner),
        }
    }

    /// Locks the Mutex and returns a guard that permits
    /// access to the inner data.
    ///
    /// The returned value may be dereferenced for data access and
    /// the lock will be dropped when the guard falls out of scope.
    pub fn lock(&self) -> MutexGuard<T> {
        self.inner.lock()
    }
}
