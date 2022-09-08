mod args;

use std::io::{self, Write};
use std::net;
use std::thread;
use std::time;

static PORT: u32 = 3000;

fn main() -> io::Result<()> {
    let args = args::parse();

    let mut stream = net::TcpStream::connect(format!("127.0.0.1:{}", PORT))?;

    for _ in 0..args.repeat {
        println!("Sending [{}]", args.message);
        stream.write_all(args.message.as_bytes())?;
        stream.write_all("\n".as_bytes())?;

        if args.delay > 0 {
            thread::sleep(time::Duration::from_millis(args.delay));
        }
    }

    stream.write("done\n".as_bytes())?;
    stream.shutdown(net::Shutdown::Both)?;

    Ok(())
}
