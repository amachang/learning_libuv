use super::{
    LOOP,
    native::*,
    error::{
        Error,
        Result,
    },
    ptr::ANonNull,
};

use std::{
    pin::Pin,
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
    sync::{
        Arc,
        Mutex,
        Weak,
    },
    ptr::NonNull,
};

use libc::{
    malloc,
    free,
};

pub struct TimerData {
    is_started: bool,
    is_elapsed: bool,
    waker: Option<Waker>,
}

pub struct Timer {
    native: ANonNull<uv_timer_t>,
    data: Arc<Mutex<TimerData>>,
}

impl Timer {
    fn try_new() -> Result<Self> {
        let native = ANonNull::new(unsafe { malloc(size_of::<uv_timer_t>()) as *mut _ });
        let Some(native) = native else {
            return Err(Error::from(io::Error::last_os_error()));
        };
        LOOP.with(|lp| {
            let lp = lp.borrow();
            let Some(lp) = lp.as_ref() else {
                return Err(Error::from("Event loop not started! Use block_on or something.".to_string()));
            };
            let r = unsafe { uv_timer_init(lp.native_ptr(), native.nn().as_ptr()) };
            if r != 0 {
                return Err(Error::from(r));
            };
            Ok(())
        })?;
        let timer = Self {
            native,
            data: Arc::new(Mutex::new(TimerData {
                is_started: false,
                is_elapsed: false,
                waker: None,
            })),
        };
        let data_weak_ref = Box::new(Arc::downgrade(&timer.data));
        unsafe { timer.native.nn().as_mut().data = Box::into_raw(data_weak_ref) as *mut _ };
        Ok(timer)
    }

    fn start_once(&mut self, duration: Duration) -> Result<()> {
        let millis = duration.as_millis();
        let millis = u64::try_from(millis).map_err(Error::from)?;
        unsafe { uv_timer_start(self.native.nn().as_ptr(), Some(cb), millis, 0u64) };

        {
            let mut data = self.data.lock().map_err(Error::from)?;
            assert!(!data.is_started && !data.is_elapsed);
            data.is_started = true;
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
    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Result<()>> {
        let mut data = self.data.lock().map_err(Error::from)?;
        assert!(data.is_started);
        if data.is_elapsed {
            Poll::Ready(Ok(()))
        } else {
            data.waker = Some(ctx.waker().clone());
            Poll::Pending
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        unsafe { free(self.native.nn().as_ptr() as *mut _) };
    }
}

pub fn sleep(duration: Duration) -> Result<Timer> {
    match Instant::now().checked_add(duration) {
        Some(_) => Timer::try_from(duration),
        None => Err(Error::from("Overflow time value".to_string())),
    }
}

extern "C" fn cb(native_ptr: *mut uv_timer_t) {
    let mut native_ptr: NonNull<uv_timer_t> = NonNull::new(native_ptr).unwrap();
    let data: Box<Weak<Mutex<TimerData>>> = unsafe { Box::from_raw(native_ptr.as_mut().data as *mut Weak<Mutex<TimerData>>) };
    let data = data.upgrade().unwrap();
    let waker = {
        let mut data = data.lock().expect("Cannot get lock in callback");
        assert!(data.is_started && !data.is_elapsed);
        data.is_elapsed = true;
        let waker = data.waker.take();
        waker
    };
    if let Some(waker) = waker {
        waker.wake();
    }
}

