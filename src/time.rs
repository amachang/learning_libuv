use super::{
    LOOP,
    native::*,
    error::{
        Error,
        Result,
    },
};

use std::{
    pin::{
        Pin,
    },
    time::{
        Duration,
        Instant,
    },
    io,
    mem::{
        size_of,
    },
    future::Future,
    task::{
        Waker,
        Poll,
        Context,
    },
    ptr::NonNull,
    thread::{
        self,
        ThreadId,
    },
};

use libc::{
    malloc,
    free,
};

pub struct TimerData {
    is_started: bool,
    is_elapsed: bool,
    waker: Option<Waker>,
    thread_id: ThreadId,
}

pub struct Timer {
    native: NonNull<uv_timer_t>,
    data: Box<TimerData>,
}

impl Timer {
    fn try_new() -> Result<Self> {
        let native = NonNull::new(unsafe { malloc(size_of::<uv_timer_t>()) as *mut _ });
        let Some(native) = native else {
            return Err(Error::from(io::Error::last_os_error()));
        };
        LOOP.with(|lp| {
            let lp = lp.borrow();
            let Some(lp) = lp.as_ref() else {
                return Err(Error::from("Event loop not started! Use block_on or something.".to_string()));
            };
            let r = unsafe { uv_timer_init(lp.native_ptr(), native.as_ptr()) };
            if r != 0 {
                return Err(Error::from(r));
            };
            Ok(())
        })?;
        let mut timer = Self {
            native,
            data: Box::new(TimerData {
                is_started: false,
                is_elapsed: false,
                waker: None,
                thread_id: thread::current().id(),
            }),
        };
        unsafe { timer.native.as_mut().data = &mut *timer.data as *mut _ as *mut _ };
        Ok(timer)
    }

    fn start_once(&mut self, duration: Duration) -> Result<()> {
        let millis = duration.as_millis();
        let millis = u64::try_from(millis).map_err(Error::from)?;
        unsafe { uv_timer_start(self.native.as_ptr(), Some(cb), millis, 0u64) };

        {
            assert!(!self.data.is_started && !self.data.is_elapsed);
            self.data.is_started = true;
        };

        Ok(())
    }
}

impl TryFrom<Duration> for Timer {
    type Error = Error;

    fn try_from(duration: Duration) -> Result<Self> {
        let mut timer = Timer::try_new()?;
        timer.start_once(duration)?;
        Ok(timer)
    }
}

impl Future for Timer {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        assert!(self.data.is_started);
        if self.data.is_elapsed {
            Poll::Ready(())
        } else {
            self.data.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        unsafe { free(self.native.as_ptr() as *mut _) };
    }
}

pub fn sleep(duration: Duration) -> Timer {
    match Instant::now().checked_add(duration) {
        Some(_) => Timer::try_from(duration).expect("Couldn't create a timer."),
        None => panic!("Overflow time value"),
    }
}

extern "C" fn cb(native_ptr: *mut uv_timer_t) {
    let mut native_ptr: NonNull<uv_timer_t> = NonNull::new(native_ptr).unwrap();
    let data: &mut TimerData = unsafe { &mut *(native_ptr.as_mut().data as *mut _) };
    if data.thread_id != thread::current().id() {
        panic!("Future must be waked in the same threads.");
    }
    let waker = {
        assert!(data.is_started && !data.is_elapsed);
        data.is_elapsed = true;
        let waker = data.waker.take();
        waker
    };
    if let Some(waker) = waker {
        waker.wake();
    }
}

