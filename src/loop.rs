use super::{
    native::*,
    error::{
        Error,
        Result,
    }
};

use std::{
    io,
    mem::{
        size_of,
    },
    ptr::NonNull,
};

use libc::{
    malloc,
    free,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RunMode {
    Default,
    Once,
    NoWait,
}

impl RunMode {
    pub fn to_native(&self) -> uv_run_mode {
        match self {
            Self::Default => uv_run_mode_UV_RUN_DEFAULT,
            Self::Once => uv_run_mode_UV_RUN_ONCE,
            Self::NoWait => uv_run_mode_UV_RUN_NOWAIT,
        }
    }
}

#[derive(Debug)]
pub struct Loop {
    native: NonNull<uv_loop_t>,
}

impl Loop {
    pub fn try_new() -> Result<Self> {
        let native = NonNull::new(unsafe { malloc(size_of::<uv_loop_t>()) as *mut _ });
        let Some(native) = native else {
            return Err(Error::from_io_error(io::Error::last_os_error()));
        };
        let r = unsafe { uv_loop_init(native.as_ptr()) };
        if r != 0 {
            return Err(Error::from_native(r));
        }
        Ok(Self { native })
    }

    pub fn run(&self, run_mode: RunMode) -> Result<()> {
        let r = unsafe { uv_run(self.native.as_ptr(), run_mode.to_native()) };
        if r != 0 {
            return Err(Error::from_native(r));
        };
        Ok(())
    }
}

impl Drop for Loop {
    fn drop(&mut self) {
        unsafe { uv_loop_close(self.native.as_ptr() as *mut _) };
        unsafe { free(self.native.as_ptr() as *mut _) };
    }
}

