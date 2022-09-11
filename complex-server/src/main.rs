mod handler;
mod logger;
mod thread_pool;

use std::io;
use std::net;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use log::{info, warn, error};

static PORT: i32 = 3000;
static THREADS: i32 = 4;

fn main() -> io::Result<()> {
    logger::setup().expect("Could not start logger");

    let thread_pool = thread_pool::ThreadPool::new(THREADS);

    info!("Created thread pool with {} threads", THREADS);

    let listener = net::TcpListener::bind(format!("127.0.0.1:{}", PORT))?;

    info!("Server started on port {}", PORT);

    let counter = Arc::new(Mutex::new(0u64));
    let uploads = Arc::new(Mutex::new(HashSet::new()));

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                let counter_clone = Arc::clone(&counter);
                let uploads_clone = Arc::clone(&uploads);
                thread_pool.execute(|| {
                    if let Some(err) = handler::handle(stream, counter_clone, uploads_clone).err() {
                        error!("{}", err);
                    };
                });
            },

            Err(err) => {
                warn!("Bad connection: {}", err);
            },
        }
    }

    Ok(())
}
