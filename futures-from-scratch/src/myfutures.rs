use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::thread;
use std::time::{Duration, Instant};

use crate::reactor::REACTOR;

#[derive(Clone)]
pub struct Timeout {
    deadline: Instant,
}

impl Timeout {
    pub fn new(duration: Duration) -> Self {
        Self {
            deadline: Instant::now() + duration,
        }
    }
}

impl Future for Timeout {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.deadline.duration_since(Instant::now()) {
            Duration::ZERO => Poll::Ready(()),
            remaning @ _ => {
                let waker = cx.waker().clone();

                thread::spawn(move || {
                    thread::sleep(remaning);
                    waker.wake();
                });

                Poll::Pending
            }
        }
    }
}

#[derive(Clone)]
pub struct SpinTimeout {
    deadline: Instant,
}

impl SpinTimeout {
    pub fn new(duration: Duration) -> Self {
        Self {
            deadline: Instant::now() + duration,
        }
    }
}

impl Future for SpinTimeout {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Duration::ZERO = self.deadline.duration_since(Instant::now()) {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[derive(Clone)]
pub struct ReactorTimeout {
    deadline: Instant,
}

impl ReactorTimeout {
    pub fn new(duration: Duration) -> Self {
        Self {
            deadline: Instant::now() + duration,
        }
    }
}

impl Future for ReactorTimeout {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Duration::ZERO = self.deadline.duration_since(Instant::now()) {
            Poll::Ready(())
        } else {
            REACTOR.lock().unwrap().register_timeout(self.deadline, cx.waker().clone());
            Poll::Pending
        }
    }
}
