mod logger;
mod reactor;
mod event_loop;
mod event_handler;
mod listener;
mod client;
mod handler;

use std::io;

use crate::event_loop::EventLoop;
use crate::listener::AsyncTcpListener;

fn main() -> io::Result<()> {
    logger::setup().unwrap();

    let mut event_loop = EventLoop::new()?;

    event_loop.register(AsyncTcpListener::bind("localhost:3000")?);

    event_loop.run()?;

    Ok(())
}
