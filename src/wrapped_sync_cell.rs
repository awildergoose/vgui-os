use core::cell::UnsafeCell;

pub struct WrappedSyncCell<T>(UnsafeCell<T>);

unsafe impl<T> Sync for WrappedSyncCell<T> {}

impl<T> WrappedSyncCell<T> {
    #[must_use]
    pub const fn new(o: T) -> Self {
        Self(UnsafeCell::new(o))
    }

    // SAFETY: This is unsound if `o` is accessed from another thread!
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        unsafe { f(&mut *self.0.get()) }
    }
}
