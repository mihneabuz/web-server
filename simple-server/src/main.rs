use std::io::{self, BufRead};
use std::net;

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

fn handle(stream: net::TcpStream) -> io::Result<usize> {
    let mut buf = String::new();
    let mut reader = io::BufReader::new(&stream);

    let (ip, port) = (
        stream.peer_addr().unwrap().ip(),
        stream.peer_addr().unwrap().port(),
    );

    info!("Connection from {}:{}", ip, port);

    loop {
        buf.clear();
        let len = reader.read_line(&mut buf)?;

        if len == 0 || buf == "done\n" {
            info!("Shutdown {}:{}", ip, port);
            stream.shutdown(net::Shutdown::Both)?;
            break;
        }

        info!("Received from {}:{} > [bytes {}][{}]", ip, port, len, &buf[..len-1]);
    }

    Ok(0)
}

fn main() -> io::Result<()> {
    setup_logger()
        .err()
        .map(|err| println!("Could not start logging: {}", err));

    let listener = net::TcpListener::bind(format!("127.0.0.1:{}", PORT))?;

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
