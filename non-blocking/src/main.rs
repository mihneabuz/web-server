mod handler;
mod logger;

use std::io::{self, Write};
use std::io::Read;
use std::net;

use std::collections::{HashSet, HashMap};

use log::{info, warn, error};
use polling::{Event, Poller};


static PORT: i32 = 3000;
static THREADS: i32 = 4;

static LISTENER: usize = 0;

fn main() -> io::Result<()> {
    logger::setup().expect("Could not start logger");

    info!("Created thread pool with {} threads", THREADS);

    let listener = net::TcpListener::bind(format!("127.0.0.1:{}", PORT))?;
    listener.set_nonblocking(true)?;

    info!("Server started on port {}", PORT);

    let poller = Poller::new()?;
    poller.add(&listener, Event::readable(LISTENER))?;
    let mut events = Vec::new();

    let mut connections = HashMap::new();
    let mut cursor = 1;
    let mut buf = [0; 256];

    let (mut counter, mut uploads) = (0u64, HashSet::new());

    loop {
        events.clear();
        poller.wait(&mut events, None)?;

        for event in events.iter() {
            info!(" -> {}", event.key);
            if event.key == LISTENER {
                match listener.accept() {
                    Ok((stream, socket)) => {
                        info!("Received connection from [{}:{}]", socket.ip(), socket.port());

                        poller.add(&stream, Event::readable(cursor))?;

                        connections.insert(cursor, stream);
                        cursor += 1;
                    },

                    Err(err) => {
                        warn!("Bad connection: {}", err);
                    },
                }

                poller.modify(&listener, Event::readable(LISTENER))?;

            } else {

                if event.readable {
                    let mut stream = connections.get(&event.key).unwrap();

                    let len = stream.read(&mut buf)?;
                    let message = String::from_utf8_lossy(&buf[..len]).to_string();
                    let res = handler::handle(message, &mut counter, &mut uploads);

                    stream.write_all(res.as_bytes())?;

                    poller.modify(stream, Event::readable(event.key))?;
                }
            }
        }
    }
}
