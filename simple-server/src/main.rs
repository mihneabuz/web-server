use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};

use fern;
use log::{error, info, warn};

static PORT: u32 = 3000;

fn setup_logger() -> Result<(), log::SetLoggerError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()
}

fn handle(mut stream: TcpStream) -> io::Result<usize> {
    let mut buf = [0u8; 256];

    stream.read(&mut buf)?;

    info!(
        "Received from {}:{} > [{}]",
        stream.peer_addr().unwrap().ip(),
        stream.peer_addr().unwrap().port(),
        std::str::from_utf8(&buf).unwrap_or("invalid string")
    );

    stream.write(&buf)?;

    Ok(0)
}

fn main() -> io::Result<()> {
    setup_logger()
        .err()
        .map(|err| println!("Could not start logging: {}", err));

    let listener = TcpListener::bind(format!("127.0.0.1:{}", PORT))?;

    info!("Server started on port {}", PORT);

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                handle(stream)
                    .err()
                    .map(|err| warn!("Stream error: {}", err));
            }
            Err(err) => {
                warn!("Bad connection: {}", err);
            }
        }
    }

    Ok(())
}
