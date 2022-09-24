use std::collections::HashMap;
use std::io::Result;

use crate::event_handler::EventHandler;
use crate::reactor::Reactor;

pub struct EventLoop {
    tasks: Vec<usize>,
    handlers: HashMap<usize, Box<dyn EventHandler>>,
    reactor: Reactor,
}

impl EventLoop {
    pub fn new() -> Result<Self> {
        Ok(Self {
            tasks: Vec::new(),
            handlers: HashMap::new(),
            reactor: Reactor::new()?,
        })
    }

    pub fn register(&mut self, handler: impl EventHandler + 'static) {
        self.tasks.push(handler.id());
        self.handlers.insert(handler.id(), Box::new(handler));
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            // 1. execute tasks
            while let Some(task) = self.tasks.pop() {
                let handler = self.handlers.get_mut(&task).unwrap();
                handler.poll(&mut self.reactor)?;
            }

            // 2. register new handlers
            while let Some(handler) = self.reactor.register.pop() {
                self.handlers.insert(handler.id(), handler);
            }

            // 3. unregister old handlers
            while let Some(key) = self.reactor.unregister.pop() {
                self.handlers.remove(&key);
            }

            // 4. handle events
            for event in self.reactor.events()? {
                let handler = self.handlers.get_mut(&event.key).unwrap();
                handler.event(event, &mut self.tasks)?;
            }
        }
    }
}
