mod executor;
mod handler;
mod logger;
mod myfutures;
mod reactor;

use std::io;
use std::time::Duration;

use futures::join;
use log::info;

use executor::block_on;
use myfutures::*;

fn main() {
    let start = std::time::Instant::now();

    let fut1 = async {
        Timeout::new(Duration::from_millis(1000)).await;
        println!("finished 1 at time: {:.2}.", start.elapsed().as_secs_f32());
    };

    let fut2 = async {
        ReactorTimeout::new(Duration::from_millis(2000)).await;
        println!("finished 2 at time: {:.2}.", start.elapsed().as_secs_f32());
    };

    let fut3 = async {
        SpinTimeout::new(Duration::from_millis(1500)).await;
        println!("finished 3 at time: {:.2}.", start.elapsed().as_secs_f32());
    };

    let mainfut = async {
        join! { fut2, fut3 };
        fut1.await;
        server().await.unwrap();
    };

    block_on(mainfut);
}

async fn server() -> io::Result<()> {
    logger::setup().unwrap();

    let listener = TcpListener::bind("127.0.0.1:3000")?;

    info!("Started TCP Listener");

    loop {
        let (stream, addr) = listener.accept().await?;
        process(stream, addr).await?;
    }
}

async fn process(mut stream: mio::net::TcpStream, addr: std::net::SocketAddr) -> io::Result<()> {
    info!("Proccessing TCP Stream");

    let mut buf = [0u8; 512];

    loop {
        let len = stream.async_read(&mut buf).await?;

        if len == 0 {
            break;
        }

        let res = handler::handle(String::from_utf8_lossy(&buf[..len]).to_string());

        stream.async_write(res.as_bytes()).await;
    }

    info!("Disconnected {}:{}", addr.ip(), addr.port());

    Ok(())
}
