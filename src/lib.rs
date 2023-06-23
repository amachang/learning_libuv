mod native;

mod error;
pub use error::*;

mod r#loop;
pub use r#loop::*;

use std::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
    sync::{
        Arc,
        RwLock,
    },
};

use futures::task::{
    waker_ref,
    ArcWake,
};

pub fn block_on<T: Send + Sync>(f: impl Future<Output = T> + Send + Sync + 'static) -> Result<T> {
    let lp = Loop::try_new()?;

    struct TaskData<T> {
        f: Pin<Box<dyn Future<Output = T> + Send + Sync + 'static>>,
        res: Result<T>,
    }
    struct Task<T> {
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
            let mut data = task.data.try_write().expect("Future couldn't get lock");
            let waker = waker_ref(&task);
            let mut ctx = Context::from_waker(&waker);

            match data.f.as_mut().poll(&mut ctx) {
                Poll::Ready(res) => {
                    data.res = Ok(res)
                },
                _ => (),
            }
        }
    }

    let task = Task::new(f);
    task.clone().wake();

    lp.run(RunMode::Default)?;

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

