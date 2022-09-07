mod args;

use std::io::{self, Write};
use std::net::TcpStream;
use std::thread;
use std::time;

static PORT: u32 = 3000;

fn main() -> io::Result<()> {
    let args = args::parse();

    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", PORT))?;

    for _ in 0..args.repeat {
        stream.write(args.message.as_bytes())?;

        if args.delay > 0 {
            thread::sleep(time::Duration::from_millis(args.delay));
        }
    }

    Ok(())
}
