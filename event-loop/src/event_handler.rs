use std::io::Result;
use crate::reactor::Reactor;
use polling::Event;

pub trait EventHandler {
    fn id(&self) -> usize;
    fn poll(&mut self, reactor: &mut Reactor) -> Result<()>;
    fn event(&mut self, event: Event, tasks: &mut Vec<usize>) -> Result<()>;
}
