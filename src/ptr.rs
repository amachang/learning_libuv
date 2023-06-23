use std::ptr::NonNull;

pub struct ANonNull<T>(NonNull<T>);

impl<T> ANonNull<T> {
    pub fn new(ptr: *mut T) -> Option<Self> {
        match NonNull::new(ptr) {
            Some(ptr) => Some(ANonNull(ptr)),
            None => None,
        }
    }

    pub fn nn(&self) -> NonNull<T> {
        self.0
    }
}

unsafe impl<T> Send for ANonNull<T> { }
unsafe impl<T> Sync for ANonNull<T> { }

