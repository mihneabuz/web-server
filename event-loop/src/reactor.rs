use std::io::Result;

use crate::event_handler::EventHandler;
use polling::{Event, Poller, Source};

pub struct Reactor {
    pub register: Vec<Box<dyn EventHandler>>,
    pub unregister: Vec<usize>,
    poller: Poller,
}

impl Reactor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            register: Vec::new(),
            unregister: Vec::new(),
            poller: Poller::new()?,
        })
    }

    pub fn add(&mut self, source: impl Source, event: Event) -> Result<()> {
        self.poller.add(source, event)
    }

    pub fn modify(&mut self, source: impl Source, event: Event) -> Result<()> {
        self.poller.modify(source, event)
    }

    pub fn remove(&mut self, source: impl Source) -> Result<()> {
        self.poller.delete(source)
    }

    pub fn register(&mut self, handler: impl EventHandler + 'static) {
        self.register.push(Box::new(handler));
    }

    pub fn unregister(&mut self, handler: &impl EventHandler) {
        self.unregister.push(handler.id());
    }

    pub fn events(&self) -> Result<std::vec::IntoIter<Event>> {
        let mut evs = Vec::new();
        self.poller.wait(&mut evs, None)?;
        Ok(evs.into_iter())
    }
}
