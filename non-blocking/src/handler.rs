use std::collections::HashSet;
use rand::{self, RngCore};

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
    "What we see is mainly what we look for.\n",
    "Silence is a source of great strength.\n",
    "Logic will get you from A to B. Imagination will take you everywhere.\n",
    "Doing your best means never stop trying.\n",
];

pub fn handle(message: String, counter: &mut u64, uploads: &mut HashSet<String>) -> String {
    match Command::parse(message.trim_end()) {
        Command::Fortune => {
            FORTUNES[rand::thread_rng().next_u32() as usize % FORTUNES.len()].to_string()
        },
        Command::Increment => {
            *counter += 1;
            "incremented\n".to_string()
        },
        Command::Counter => {
            format!("counter: {}\n", *counter)
        },
        Command::Upload(item) => {
            uploads.insert(item);
            "uploaded\n".to_string()
        },
        Command::Download(item) => {
            let found = uploads.get(&item).cloned().unwrap_or_else(|| String::from("not found"));
            format!("download: {}\n", found)
        },
        Command::None => {
            "ok\n".to_string()
        }
    }
}
