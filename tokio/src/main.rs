mod handler;
mod logger;

use std::{io::Result, net::SocketAddr};

use log::{info, warn};
use tokio::{net::{TcpListener, TcpStream}, io::{AsyncReadExt, AsyncWriteExt}};

#[tokio::main]
async fn main() -> Result<()> {
    logger::setup().unwrap();

    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    loop {
        let (stream, addr) = listener.accept().await?;
        tokio::spawn(async move {
            info!("Connection from {}:{}", addr.ip(), addr.port());
            process(stream, addr).await.unwrap_or_else(|err| warn!("Error: {}", err));
        });
    }
}

async fn process(mut stream: TcpStream, addr: SocketAddr) -> Result<()> {
    let mut buf = [0u8; 512];

    loop {
        let len = stream.read(&mut buf).await?;

        if len == 0 {
            break;
        }

        let res = handler::handle(String::from_utf8_lossy(&buf[..len]).to_string());

        stream.write_all(res.as_bytes()).await?;
    }

    info!("Disconnected {}:{}", addr.ip(), addr.port());

    Ok(())
}
