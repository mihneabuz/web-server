mod handler;
mod logger;
mod thread_pool;

use std::io;
use std::net;

use log::{info, warn};

static PORT: i32 = 3000;
static THREADS: i32 = 4;

fn main() -> io::Result<()> {
    logger::setup().expect("Could not start logger");

    let thread_pool = thread_pool::ThreadPool::new(THREADS);

    info!("Created thread pool with {} threads", THREADS);

    let listener = net::TcpListener::bind(format!("127.0.0.1:{}", PORT))?;

    info!("Server started on port {}", PORT);

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                thread_pool.execute(|| {
                    handler::handle(stream);
                });
            }
            Err(err) => {
                warn!("Bad connection: {}", err);
            }
        }
    }

    Ok(())
}
