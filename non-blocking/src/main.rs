mod handler;
mod logger;

use std::io::{self, Write};
use std::io::Read;
use std::net;

use std::os::unix::io::AsRawFd;

use std::collections::{HashSet, HashMap};

use log::{info, warn};
use polling::{Event, Poller};

static PORT: i32 = 3000;
static THREADS: i32 = 4;

struct Connection {
    stream: net::TcpStream,
    response: Option<String>
}

fn main() -> io::Result<()> {
    logger::setup().expect("Could not start logger");

    info!("Created thread pool with {} threads", THREADS);

    let listener = net::TcpListener::bind(format!("127.0.0.1:{}", PORT))?;
    let listener_id = listener.as_raw_fd() as usize;
    listener.set_nonblocking(true)?;

    info!("Server started on port {}", PORT);

    let poller = Poller::new()?;
    poller.add(&listener, Event::readable(listener_id))?;
    let mut events = Vec::new();

    let mut connections = HashMap::new();
    let mut buf = [0; 256];

    let (mut counter, mut uploads) = (0u64, HashSet::new());

    let mut iter = 0;
    loop {
        events.clear();
        poller.wait(&mut events, None)?;

        if events.len() > 1 {
            info!("Event loop iter [{}] -> events [{}]", iter, events.len());
        }
        iter += 1;

        for ev in events.iter() {
            if ev.key == listener_id && ev.readable {
                match listener.accept() {
                    Ok((stream, socket)) => {
                        info!("Received connection from [{}:{}]", socket.ip(), socket.port());

                        let connection_fd = stream.as_raw_fd() as usize;
                        poller.add(&stream, Event::readable(connection_fd))?;
                        connections.insert(connection_fd, Connection{ stream, response: None});
                    },

                    Err(err) => {
                        warn!("Bad connection: {}", err);
                    },
                }

                poller.modify(&listener, Event::readable(listener_id))?;

            } else {

                if ev.readable {
                    let conn = connections.get_mut(&ev.key).unwrap();

                    let len = conn.stream.read(&mut buf)?;
                    if len > 0 {
                        let message = String::from_utf8_lossy(&buf[..len]).to_string();
                        conn.response = Some(handler::handle(message, &mut counter, &mut uploads));
                        poller.modify(&conn.stream, Event::writable(ev.key))?;
                    } else {
                        poller.delete(&conn.stream)?;
                        connections.remove(&ev.key);
                    }

                } else if ev.writable {
                    let conn = connections.get_mut(&ev.key).unwrap();

                    match &conn.response {
                        Some(res) => {
                            conn.stream.write_all(res.as_bytes())?;
                        },

                        None => {
                            warn!("Response not created!");
                        }
                    }

                    poller.modify(&conn.stream, Event::readable(ev.key))?;
                }
            }
        }
    }
}
