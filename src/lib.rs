mod native;

pub mod ptr;

mod error;
pub use error::*;

mod r#loop;
pub use r#loop::*;

pub mod time;

use std::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
    cell::RefCell,
    sync::{
        Arc,
        RwLock,
    },
};

use futures::task::{
    waker_ref,
    ArcWake,
};

thread_local!(pub static LOOP: RefCell<Option<Loop>> = RefCell::new(None));

pub struct TaskData<T> {
    f: Pin<Box<dyn Future<Output = T> + Send + Sync + 'static>>,
    res: Result<T>,
}

pub struct Task<T> {
    data: RwLock<TaskData<T>>
}

impl<T> Task<T> {
    fn new(f: impl Future<Output = T> + Send + Sync + 'static) -> Arc<Self> {
        Arc::new(Self {
            data: RwLock::new(TaskData {
                f: Box::pin(f),
                res: Err(Error::from("Result not set in a future".to_string())),
            }),
        })
    }
}

impl<T: Send + Sync> ArcWake for Task<T> {
    fn wake_by_ref(task: &Arc<Self>) {
        let waker = waker_ref(&task);
        let mut ctx = Context::from_waker(&waker);

        let mut data = task.data.try_write().expect("Future couldn't get lock");
        match data.f.as_mut().poll(&mut ctx) {
            Poll::Ready(res) => {
                data.res = Ok(res)
            },
            _ => (),
        }
    }
}

pub fn block_on<T: Send + Sync>(f: impl Future<Output = T> + Send + Sync + 'static) -> Result<T> {

    let already_started = LOOP.with(|lp| !lp.borrow().is_none());
    if already_started {
        return Err(Error::from("Loop already started in this thread.".to_string()));
    };

    let new_lp = Loop::try_new()?;
    LOOP.with(move |lp| lp.borrow_mut().replace(new_lp));

    let task = Task::new(f);
    task.clone().wake();

    LOOP.with(|lp| {
        let res = match lp.borrow().as_ref() {
            Some(lp) => lp.run(RunMode::Default),
            None => Err(Error::from("Unexpected none of loop".to_string())),
        };
        lp.borrow_mut().take();
        res
    })?;

    match Arc::try_unwrap(task) {
        Ok(task) => {
            Ok(task.data.into_inner().map_err(Error::from)?.res?)
        },
        Err(task) => {
            Err(Error::from(format!(
                        "Remains ref count for future result, circular reference or early exiting event loop?: (ref_count = {})",
                        Arc::strong_count(&task),
            )))
        }
    }
}

