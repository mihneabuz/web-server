use std::task::{Waker, Poll, Context};
use std::future::Future;
use std::pin::Pin;

use crate::reactor::REACTOR;

#[derive(Clone)]
pub struct Task {
    id: usize,
    data: u64,
}

pub enum TaskState {
    Ready,
    NotReady(Waker),
    Finished,
}

impl Task {
    pub fn new(data: u64, id: usize) -> Self {
        Task { id, data }
    }
}

impl Future for Task {
    type Output = usize;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut r = REACTOR.lock().unwrap();
        if r.is_ready(self.id) {
            *r.tasks.get_mut(&self.id).unwrap() = TaskState::Finished;
            Poll::Ready(self.id)
        } else if let std::collections::hash_map::Entry::Occupied(mut e) = r.tasks.entry(self.id) {
            e.insert(TaskState::NotReady(cx.waker().clone()));
            Poll::Pending
        } else {
            r.register(self.data, cx.waker().clone(), self.id);
            Poll::Pending
        }
    }
}
