use std::cmp::Reverse;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::task::Waker;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{thread, time};

use mio::event::Source;
use mio::{Events, Interest, Poll, Token};
use once_cell::sync::Lazy;

pub static REACTOR: Lazy<Mutex<Box<Reactor>>> = Lazy::new(|| Reactor::new());

pub struct Reactor {
    timeout_handler: TimeoutHandler,
    io_handler: IOHandler,
}

impl Reactor {
    fn new() -> Mutex<Box<Self>> {
        Mutex::new(Box::new(Reactor {
            timeout_handler: TimeoutHandler::new(),
            io_handler: IOHandler::new(),
        }))
    }

    pub fn register_timeout(&self, deadline: time::Instant, waker: Waker) {
        self.timeout_handler.register_timeout(deadline, waker);
    }

    pub fn register_read<S: Source>(&self, source: &mut S, waker: Waker) {
        self.io_handler
            .register_operation(source, Interest::READABLE, waker)
    }

    pub fn register_write<S: Source>(&self, source: &mut S, waker: Waker) {
        self.io_handler
            .register_operation(source, Interest::WRITABLE, waker)
    }

    pub fn deregister<S: Source>(&self, source: &mut S) {
        self.io_handler.deregister_operation(source);
    }
}

struct TimeoutHandler {
    join_handle: Option<JoinHandle<()>>,
    dispatcher: Sender<TimeoutEvent>,
}

#[derive(Debug)]
enum TimeoutEvent {
    Close,
    Signal(time::Instant, Waker),
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
                        }
                        TimeoutEvent::Close => {
                            println!("[killed timeout handler]");
                            return;
                        }
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
        self.dispatcher
            .send(TimeoutEvent::Signal(deadline, waker))
            .unwrap();
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
    id_counter: AtomicUsize,
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
                    println!("[killed io handler]");
                    return;
                }

                std::thread::sleep(Duration::from_millis(10));
                {
                    let mut poll = poll.lock().unwrap();

                    poll.poll(&mut events, Some(Duration::from_millis(10)))
                        .unwrap();

                    for event in &events {
                        let Token(id) = event.token();
                        if let Some(waker) = { wakers.lock().unwrap().remove(&id) } {
                            waker.wake();
                        }
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

    fn register_waker(&self, waker: Waker) -> usize {
        let id = self.id_counter.fetch_add(1, Ordering::SeqCst);
        self.wakers.lock().unwrap().insert(id, waker);
        id
    }

    fn register_operation<S: Source>(&self, source: &mut S, interest: Interest, waker: Waker) {
        let id = self.register_waker(waker);

        let poll = self.poll.lock().unwrap();
        let registry = poll.registry();
        registry.register(source, Token(id), interest).unwrap();
    }

    fn deregister_operation<S: Source>(&self, source: &mut S) {
        let poll = self.poll.lock().unwrap();
        let registry = poll.registry();
        registry.deregister(source);
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
