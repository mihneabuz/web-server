use std::net::TcpStream;
use std::io::{Result, Read, Write};
use std::os::unix::prelude::AsRawFd;

use log::info;
use polling::Event;

use crate::event_handler::EventHandler;
use crate::handler;
use crate::reactor::Reactor;

enum State {
    WaitingRead,
    Reading,
    WaitingWrite,
    Writing,
    Finished
}

pub struct AsyncClientHandler {
    stream: TcpStream,
    state: State,
    response: Option<String>
}

impl AsyncClientHandler {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream, state: State::WaitingRead, response: None }
    }
}

impl EventHandler for AsyncClientHandler {
    fn id(&self) -> usize {
        self.stream.as_raw_fd() as usize
    }

    fn poll(&mut self, reactor: &mut Reactor) -> Result<()> {
        match self.state {
            State::Reading => {
                let mut buf = [0u8; 512];
                let len = self.stream.read(&mut buf)?;

                if len == 0 {
                    info!("Client disconnected!");
                    reactor.remove(&self.stream)?;
                    reactor.unregister(self);
                    self.state = State::Finished;
                } else {
                    self.response.replace(handler::handle(String::from_utf8_lossy(&buf[..len]).to_string()));

                    reactor.modify(&self.stream, Event::writable(self.id()))?;
                    self.state = State::WaitingWrite;
                }
            }

            State::Writing => {
                self.stream.write_all(self.response.take().unwrap().as_bytes())?;

                reactor.modify(&self.stream, Event::readable(self.id()))?;
                self.state = State::WaitingRead;
            }

            _ => {}
        }

        Ok(())
    }

    fn event(&mut self, event: polling::Event, tasks: &mut Vec<usize>) -> Result<()> {
        match self.state {
            State::WaitingRead => {
                if event.readable {
                    tasks.push(self.id());
                    self.state = State::Reading;
                }
            },

            State::WaitingWrite => {
                if event.writable {
                    tasks.push(self.id());
                    self.state = State::Writing;
                }
            },

            _ => {}
        }

        Ok(())
    }
}
