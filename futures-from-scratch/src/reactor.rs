use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::task::Waker;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{thread, time};
use std::cmp::Reverse;

use mio::event::Source;
use mio::{Events, Poll, Interest, Token};
use once_cell::sync::Lazy;

pub static REACTOR: Lazy<Mutex<Box<Reactor>>> = Lazy::new(|| {
    Reactor::new()
});

pub struct Reactor {
    timeout_handler: TimeoutHandler,
}

impl Reactor {
    fn new() -> Mutex<Box<Self>> {
        Mutex::new(Box::new(Reactor {
            timeout_handler: TimeoutHandler::new()
        }))
    }

    pub fn register_timeout(&self, deadline: time::Instant, waker: Waker) {
        self.timeout_handler.register_timeout(deadline, waker);
    }
}

struct TimeoutHandler {
    join_handle: Option<JoinHandle<()>>,
    dispatcher: Sender<TimeoutEvent>
}

#[derive(Debug)]
enum TimeoutEvent {
    Close,
    Signal(time::Instant, Waker)
}

impl TimeoutHandler {
    pub fn new() -> Self {
        let (tx, rx) = channel::<TimeoutEvent>();

        let handle = thread::spawn(move || {
            println!("[spawned timeout handler]");
            let mut timeouts = Vec::new();

            loop {
                while let Ok(ev) = rx.try_recv() {
                    match ev {
                        TimeoutEvent::Signal(dl, waker) => {
                            timeouts.push((dl, waker));
                        },
                        TimeoutEvent::Close => {
                            return;
                        },
                    }
                }

                timeouts.sort_by_key(|t| Reverse(t.0));

                let mut should_yield = true;
                if let Some((dl, waker)) = timeouts.last() {
                    if time::Instant::now() > *dl {
                        waker.wake_by_ref();
                        should_yield = false;
                    }
                }

                if should_yield {
                    thread::yield_now();
                } else {
                    timeouts.pop();
                }
            }
        });

        Self {
            join_handle: Some(handle),
            dispatcher: tx,
        }
    }

    pub fn register_timeout(&self, deadline: time::Instant, waker: Waker) {
        self.dispatcher.send(TimeoutEvent::Signal(deadline, waker)).unwrap();
    }
}

impl Drop for TimeoutHandler {
    fn drop(&mut self) {
        self.dispatcher.send(TimeoutEvent::Close).unwrap();
        if let Some(handle) = self.join_handle.take() {
            handle.join().unwrap();
        }
    }
}

struct IOHandler {
    join_handle: Option<JoinHandle<()>>,
    poll: Arc<Mutex<Poll>>,
    wakers: Arc<Mutex<HashMap<usize, Waker>>>,
    exit: Sender<()>,
    id_counter: AtomicUsize
}

impl IOHandler {
    fn new() -> Self {
        let (tx, rx) = channel::<()>();

        let poll = Arc::new(Mutex::new(Poll::new().unwrap()));
        let wakers = Arc::new(Mutex::new(HashMap::<usize, Waker>::new()));

        let poll_clone = Arc::clone(&poll);
        let wakers_clone = Arc::clone(&wakers);

        let handle = thread::spawn(move || {
            println!("[spawned io handler]");
            let poll = poll_clone;
            let wakers = wakers_clone;

            let mut events = Events::with_capacity(1024);

            loop {
                if let Ok(()) = rx.try_recv() {
                    return;
                }

                {
                    poll.lock().unwrap()
                        .poll(&mut events, Some(Duration::from_millis(10)))
                        .unwrap();
                }

                for event in &events {
                    let Token(id) = event.token();
                    if let Some(waker) = { wakers.lock().unwrap().remove(&id) } {
                        waker.wake();
                    }
                }
            }
        });

        Self {
            join_handle: Some(handle),
            poll,
            wakers,
            exit: tx,
            id_counter: AtomicUsize::new(0),
        }
    }

    fn register_read<S: Source>(&self, source: &mut S, waker: Waker) {
        let id = self.id_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        {
            self.wakers.lock().unwrap()
                .insert(id, waker);
        }

        self.poll.lock().unwrap()
            .registry()
            .register(source, Token(0), Interest::READABLE)
            .unwrap();
    }
}

impl Drop for IOHandler {
    fn drop(&mut self) {
        self.exit.send(()).unwrap();
        if let Some(handle) = self.join_handle.take() {
            handle.join().unwrap();
        }
    }
}
