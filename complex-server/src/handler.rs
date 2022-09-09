use std::io::BufRead;
use std::net;
use std::io;

use log::info;

pub fn handle(stream: net::TcpStream) -> io::Result<()> {
    let (ip, port) = (
        stream.peer_addr().unwrap().ip(),
        stream.peer_addr().unwrap().port(),
    );

    let mut reader = io::BufReader::new(stream);
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

        info!("Received from {}:{} > [bytes {}][{}]", ip, port, len, &buf[..len-1]);
    }

    Ok(())
}
