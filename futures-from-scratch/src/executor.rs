use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Condvar, Mutex};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

#[derive(Clone)]
struct MyWaker {
    parker: Arc<Parker>,
}

fn mywaker_wake(s: &MyWaker) {
    let waker_arc = unsafe { Arc::from_raw(s as *const MyWaker) };
    waker_arc.parker.unpark();
}

fn mywaker_clone(s: &MyWaker) -> RawWaker {
    let arc = unsafe { Arc::from_raw(s) };
    std::mem::forget(arc.clone()); // increase ref count
    RawWaker::new(Arc::into_raw(arc) as *const (), &VTABLE)
}

const VTABLE: RawWakerVTable = unsafe {
    RawWakerVTable::new(
        |s| mywaker_clone(&*(s as *const MyWaker)),   // clone
        |s| mywaker_wake(&*(s as *const MyWaker)),    // wake
        |s| (*(s as *const MyWaker)).parker.unpark(), // wake by ref (don't decrease refcount)
        |s| drop(Arc::from_raw(s as *const MyWaker)), // decrease refcount
    )
};

fn mywaker_into_waker(s: *const MyWaker) -> Waker {
    let raw_waker = RawWaker::new(s as *const (), &VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}

#[derive(Default)]
struct Parker {
    resumable: Mutex<bool>,
    condvar: Condvar,
}

impl Parker {
    fn park(&self) {
        let mut resumable = self.resumable.lock().unwrap();
        while !*resumable {
            resumable = self.condvar.wait(resumable).unwrap();
        }
        *resumable = false;
    }

    fn unpark(&self) {
        *self.resumable.lock().unwrap() = true;
        self.condvar.notify_one();
    }
}

pub fn block_on<F: Future>(mut future: F) -> F::Output {
    let parker = Arc::new(Parker::default());
    let mywaker = Arc::new(MyWaker {
        parker: parker.clone(),
    });
    let waker = mywaker_into_waker(Arc::into_raw(mywaker));
    let mut cx = Context::from_waker(&waker);

    // SAFETY: we shadow `future` so it can't be accessed again.
    let mut future = unsafe { Pin::new_unchecked(&mut future) };
    loop {
        match Future::poll(future.as_mut(), &mut cx) {
            Poll::Ready(val) => break val,
            Poll::Pending => parker.park(),
        };
    }
}
