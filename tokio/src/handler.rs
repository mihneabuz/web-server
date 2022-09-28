use std::collections::HashSet;
use rand::{self, RngCore};

enum Command {
    Fortune,
    Increment,
    Counter,
    Upload(String),
    Download(String),
    Compute(u64),
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

                if other.starts_with("compute") {
                    if let Some(split) = other.split_once(' ') {
                        return Command::Compute(split.1.parse().unwrap_or(0));
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

static mut COUNTER: u64 = 0;
static mut UPLOADS: Option<HashSet<String>> = None;

fn is_prime(x: u64) -> bool {
    if x == 0 || x == 1 {
        return false;
    }

    for i in 2..x/2 {
        if x % i == 0 {
            return false;
        }
    }

    true
}

pub fn handle(message: String) -> String {
    unsafe {
        if UPLOADS == None {
            UPLOADS = Some(HashSet::new());
        }
    }

    match Command::parse(message.trim_end()) {
        Command::Fortune => {
            FORTUNES[rand::thread_rng().next_u32() as usize % FORTUNES.len()].to_string()
        },
        Command::Increment => {
            unsafe {
                COUNTER += 1;
            }
            "incremented\n".to_string()
        },
        Command::Counter => {
            format!("counter: {}\n", unsafe { COUNTER })
        },
        Command::Upload(item) => {
            unsafe {
                UPLOADS.as_mut().map(|set| set.insert(item));
            }
            "uploaded\n".to_string()
        },
        Command::Download(item) => {
            let found = unsafe {
                UPLOADS.as_ref().map(|set| set.get(&item).cloned().unwrap_or_else(|| String::from("not found"))).unwrap()
            };
            format!("download: {}\n", found)
        },
        Command::Compute(k) => {
            let sum = (0..=k).fold(0, |acc, x| acc + if is_prime(x) { x } else { 0 });
            format!("computed: {}\n", sum)
        },
        Command::None => {
            "ok\n".to_string()
        }
    }
}
