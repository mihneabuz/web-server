use std::io::Result;
use std::mem;
use std::net::{TcpListener, TcpStream};
use std::os::unix::prelude::AsRawFd;

use crate::client::AsyncClientHandler;
use crate::event_handler::EventHandler;
use crate::reactor::Reactor;

use log::info;
use polling::Event;

enum State {
    Started,
    Waiting,
    Accepting(TcpStream),
}

pub struct AsyncTcpListener {
    listener: TcpListener,
    state: State,
}

impl AsyncTcpListener {
    pub fn bind(addr: &str) -> Result<Self> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        Ok(Self {
            listener,
            state: State::Started,
        })
    }
}

impl EventHandler for AsyncTcpListener {
    fn id(&self) -> usize {
        self.listener.as_raw_fd() as usize
    }

    fn poll(&mut self, reactor: &mut Reactor) -> Result<()> {
        match mem::replace(&mut self.state, State::Waiting) {
            State::Started => {
                reactor.add(&self.listener, Event::readable(self.id()))?;
            },

            State::Accepting(stream) => {
                reactor.add(&stream, Event::readable(stream.as_raw_fd() as usize))?;
                reactor.register(AsyncClientHandler::new(stream));

                reactor.modify(&self.listener, Event::readable(self.id()))?;
            },

            _ => {}
        }

        Ok(())
    }

    fn event(&mut self, event: Event, tasks: &mut Vec<usize>) -> Result<()> {
        match self.state {
            State::Waiting => {
                if event.readable {
                    info!("Client connected!");
                    let (stream, _) = self.listener.accept()?;
                    tasks.push(self.id());
                    self.state = State::Accepting(stream);
                }
            },

            _ => {}
        }

        Ok(())
    }
}
