mod native;

mod error;
pub use error::*;

mod r#loop;
pub use r#loop::*;

pub mod time;

use std::{
    future::Future,
    pin::{
        pin,
        Pin,
    },
    task::{
        Context,
        Poll,
        Waker,
        RawWaker,
        RawWakerVTable,
    },
    cell::RefCell,
    thread::{
        self,
        ThreadId,
    }
};

struct Task<'a, F: Future>(Pin<&'a mut F>, Option<F::Output>, ThreadId);

thread_local!(pub static LOOP: RefCell<Option<Loop>> = RefCell::new(None));

pub fn block_on<F>(f: F) -> F::Output
where
    F: Future
{
    let already_started = LOOP.with(|lp| !lp.borrow().is_none());
    if already_started {
        panic!("Loop already started in this thread.");
    };

    let new_lp = Loop::try_new().expect("Couldn't initialize event loop.");
    LOOP.with(move |lp| lp.borrow_mut().replace(new_lp));

    let f = pin!(f);
    let task = Task(f, None, thread::current().id());

    wake::<F>(&task as *const _ as *const _);

    LOOP.with(|lp| {
        lp.borrow().as_ref().expect("Couldn't borrow the event loop.").run(RunMode::Default).expect("Couldn't run the event loop.");
        lp.borrow_mut().take();
    });

    let Task(_, ret, _) = task;
    ret.expect("Couldn't get the future result")
}

fn new_raw_waker<F>(task_ptr: *const ()) -> RawWaker
where
    F: Future
{
    fn noop(_: *const()) { }
    let raw_waker_vtable: &'static _ = &RawWakerVTable::new(
        new_raw_waker::<F>,
        wake::<F>,
        wake::<F>,
        noop,
    );
    RawWaker::new(task_ptr, raw_waker_vtable)
}

fn wake<F>(task_ptr: *const())
where
    F: Future
{
    let task: &mut Task<F> = unsafe { &mut *(task_ptr as *mut _) };
    let Task(ref mut f, ref mut result, thread_id) = task;

    if *thread_id != thread::current().id() {
        panic!("Future must be polled in the same threads.");
    }

    let raw_waker = new_raw_waker::<F>(task_ptr);
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut cx = Context::from_waker(&waker);

    if let Poll::Ready(r) = f.as_mut().poll(&mut cx) {
        *result = Some(r);
    };
}

