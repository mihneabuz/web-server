use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use once_cell::sync::Lazy;

pub static REACTOR: Lazy<Arc<Mutex<Box<Reactor>>>> = Lazy::new(|| {
    Reactor::new()
});

pub struct Reactor {
    dispatcher: Sender<Event>,
}

#[derive(Debug)]
enum Event {
    Close,
}

impl Reactor {
    fn new() -> Arc<Mutex<Box<Self>>> {
        let (tx, rx) = channel::<Event>();

        let reactor = Arc::new(Mutex::new(Box::new(Reactor {
            dispatcher: tx,
        })));

        let reactor_clone = Arc::downgrade(&reactor);
        let handle = thread::spawn(move || {
            while let Ok(ev) = rx.recv() {
                match ev {
                    Event::Close => return,
                }
            }
        });

        reactor
    }
}

impl Drop for Reactor {
    fn drop(&mut self) {
        self.dispatcher.send(Event::Close).unwrap();
    }
}
