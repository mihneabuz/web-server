use std::net;
use std::io;

use std::collections::HashSet;
use std::io::{BufRead, Write};
use std::sync::{Arc, Mutex};

use rand::{self, RngCore};
use log::info;

enum Command {
    Fortune,
    Increment,
    Counter,
    Upload(String),
    Download(String),
    None
}

impl Command {
    fn parse(str: &str) -> Self {
        match str {
            "fortune" => Command::Fortune,
            "increment" => Command::Increment,
            "counter" => Command::Counter,
            other => {
                if other.starts_with("upload") {
                    if let Some(split) = other.split_once(' ') {
                        return Command::Upload(String::from(split.1));
                    }
                }

                if other.starts_with("download") {
                    if let Some(split) = other.split_once(' ') {
                        return Command::Download(String::from(split.1));
                    }
                }

                Command::None
            }
        }
    }
}

static FORTUNES: &[&str] = &[
    "What we see is mainly what we look for.",
    "Silence is a source of great strength.",
    "Logic will get you from A to B. Imagination will take you everywhere.",
    "Doing your best means never stop trying.",
];

type Counter = Arc<Mutex<u64>>;
type Uploads = Arc<Mutex<HashSet<String>>>;

pub fn handle(stream: net::TcpStream, counter: Counter, uploads: Uploads) -> io::Result<()> {
    let (ip, port) = (
        stream.peer_addr().unwrap().ip(),
        stream.peer_addr().unwrap().port(),
    );

    let mut reader = io::BufReader::new(&stream);
    let mut writer = io::BufWriter::new(&stream);
    let mut buf = String::new();

    info!("Connection from {}:{}", ip, port);

    loop {
        buf.clear();
        let len = reader.read_line(&mut buf)?;

        if len == 0 || buf == "done\n" {
            info!("Shutdown {}:{}", ip, port);
            reader.into_inner().shutdown(net::Shutdown::Both)?;
            break;
        }

        match Command::parse(&buf) {
            Command::Fortune => {
                writer.write_all(FORTUNES[rand::thread_rng().next_u32() as usize % FORTUNES.len()].as_bytes())?;
            },
            Command::Increment => {
                let mut value = counter.lock().unwrap();
                *value += 1;
                writer.write_all("incremented".as_bytes())?;
            },
            Command::Counter => {
                let value = counter.lock().unwrap();
                writer.write_all(format!("counter: {}", value).as_bytes())?;
            },
            Command::Upload(item) => {
                uploads.lock().unwrap().insert(item);
                writer.write_all("uploaded".as_bytes())?;
            },
            Command::Download(item) => {
                let found = uploads.lock().unwrap().get(&item).cloned().unwrap_or_else(|| String::from("not found"));
                writer.write_all(format!("download: {}", found).as_bytes())?;
            },
            Command::None => {
                writer.write_all("ok".as_bytes())?;
            },
        }
    }

    Ok(())
}
