mod handler;
mod logger;
mod thread_pool;

use std::io::{self, Write, Read};
use std::net;

use std::collections::HashSet;
use std::os::unix::io::AsRawFd;
use std::sync::{Arc, Mutex};

use log::{info, warn};
use polling::{Event, Poller};

use crate::thread_pool::ThreadPool;

static PORT: i32 = 3000;
static THREADS: i32 = 4;

struct State {
    listener: net::TcpListener,
    listener_id: usize,

    responses: Arc<Mutex<Vec<Option<String>>>>,
    connections: Vec<Option<net::TcpStream>>,

    poller: Poller,
    events: Vec<Event>,

    counter: Arc<Mutex<u64>>,
    uploads: Arc<Mutex<HashSet<String>>>,
}

fn main() -> io::Result<()> {
    logger::setup().expect("Could not start logger");

    info!("Created thread pool with {} threads", THREADS);

    let listener = net::TcpListener::bind(format!("127.0.0.1:{}", PORT))?;
    let listener_id = listener.as_raw_fd() as usize;
    listener.set_nonblocking(true)?;

    info!("Server started on port {}", PORT);

    let thread_pool = ThreadPool::new(4);

    let mut state = State {
        listener,
        listener_id,

        connections: Vec::new(),
        responses: Arc::new(Mutex::new(Vec::new())),

        events: Vec::new(),
        poller: Poller::new()?,

        counter: Arc::new(Mutex::new(0u64)),
        uploads: Arc::new(Mutex::new(HashSet::new())),
    };

    state.poller.add(&state.listener, Event::readable(state.listener_id))?;

    let mut iter = 0;

    loop {
        state.events.clear();
        state.poller.wait(&mut state.events, None)?;

        if state.events.len() > 1 {
            info!("Event loop iter [{}] -> events [{}]", iter, state.events.len());
        }
        iter += 1;

        for ev in state.events.iter() {
            if ev.key == state.listener_id && ev.readable {
                match state.listener.accept() {
                    Ok((stream, socket)) => {
                        info!("Received connection from [{}:{}]", socket.ip(), socket.port());
                        stream.set_nonblocking(true)?;

                        let connection_fd = stream.as_raw_fd() as usize;
                        state.poller.add(&stream, Event::readable(connection_fd))?;

                        let mut locked_responses = state.responses.lock().unwrap();
                        while state.connections.len() <= connection_fd {
                            state.connections.push(None);
                            locked_responses.push(None);
                        }

                        state.connections[connection_fd] = Some(stream);
                        locked_responses[connection_fd] = None;
                    },

                    Err(err) => {
                        warn!("Bad connection: {}", err);
                    },
                }

                state.poller.modify(&state.listener, Event::readable(state.listener_id))?;

            } else {

                if ev.readable {
                    let conn = state.connections.get_mut(ev.key).unwrap().as_mut().unwrap();

                    let mut buf = [0; 256];
                    let len = conn.read(&mut buf)?;
                    if len > 0 {
                        let message = String::from_utf8_lossy(&buf[..len]).to_string();

                        let responses = Arc::clone(&state.responses);
                        let key = ev.key;
                        let (mut counter, mut uploads) = (Arc::clone(&state.counter), Arc::clone(&state.uploads));

                        thread_pool.execute(move || {
                            responses.lock().unwrap()[key] = Some(handler::handle(message, &mut counter, &mut uploads));
                        });

                        state.poller.modify(&*conn, Event::writable(ev.key))?;
                    } else {
                        state.poller.delete(&*conn)?;
                    }

                } else if ev.writable {
                    let conn = state.connections.get_mut(ev.key).unwrap().as_mut().unwrap();

                    let response = {
                        state.responses.lock().unwrap().get_mut(ev.key).unwrap().take()
                    };

                    match response {
                        Some(res) => {
                            conn.write_all(res.as_bytes())?;
                            state.poller.modify(&*conn, Event::readable(ev.key))?;
                        },

                        None => {
                            state.poller.modify(&*conn, Event::writable(ev.key))?;
                        }
                    }
                }
            }
        }
    }
}
