use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Timeout {
    deadline: Instant,
}

impl Timeout {
    pub fn new(duration: Duration) -> Self {
        Timeout {
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
