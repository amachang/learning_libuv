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

struct TaskData<'a, T, F>
where
    F: Future<Output = T> + Send + Sync + 'static
{
    f: Pin<&'a mut F>,
    res: Result<T>,
}

struct Task<'a, T, F>
where
    F: Future<Output = T> + Send + Sync + 'static
{
    data: RwLock<TaskData<'a, T, F>>
}

impl<'a, T, F> Task<'a, T, F>
where
    F: Future<Output = T> + Send + Sync + 'static
{
    fn new(f: Pin<&'a mut F>) -> Arc<Self> {
        Arc::new(Self {
            data: RwLock::new(TaskData {
                f,
                res: Err(Error::from("Result not set in a future".to_string())),
            }),
        })
    }
}

impl<'a, T, F> ArcWake for Task<'a, T, F>
where
    T: Send + Sync,
    F: Future<Output = T> + Send + Sync + 'static
{
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

pub fn block_on<T, F>(f: F) -> T
where
    T: Send + Sync,
    F: Future<Output = T> + Send + Sync + 'static
{
    let already_started = LOOP.with(|lp| !lp.borrow().is_none());
    if already_started {
        panic!("Loop already started in this thread.");
    };

    let new_lp = Loop::try_new().expect("Couldn't initialize event loop.");
    LOOP.with(move |lp| lp.borrow_mut().replace(new_lp));

    let f = std::pin::pin!(f);
    let task = Task::new(f);
    task.clone().wake();

    LOOP.with(|lp| {
        lp.borrow().as_ref().expect("Couldn't borrow the event loop.").run(RunMode::Default).expect("Couldn't run the event loop.");
        lp.borrow_mut().take();
    });

    match Arc::try_unwrap(task) {
        Ok(task) => {
            task.data.into_inner().expect("Couldn't move result").res.expect("Couldn't get future result")
        },
        Err(_) => {
            panic!("Remains ref count for future result, circular reference or early exiting event loop?");
        }
    }
}

