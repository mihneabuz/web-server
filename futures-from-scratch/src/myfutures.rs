use std::future::Future;
use std::io::{self, ErrorKind};
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::thread;
use std::time::{Duration, Instant};

use mio::event::Source;
use mio::net::TcpStream;

use crate::reactor::REACTOR;

#[derive(Clone)]
pub struct Timeout {
    deadline: Instant,
}

impl Timeout {
    pub fn new(duration: Duration) -> Self {
        Self {
            deadline: Instant::now() + duration,
        }
    }
}

impl Future for Timeout {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.deadline.duration_since(Instant::now()) {
            Duration::ZERO => Poll::Ready(()),
            remaning @ _ => {
                let waker = cx.waker().clone();

                thread::spawn(move || {
                    thread::sleep(remaning);
                    waker.wake();
                });

                Poll::Pending
            }
        }
    }
}

#[derive(Clone)]
pub struct SpinTimeout {
    deadline: Instant,
}

impl SpinTimeout {
    pub fn new(duration: Duration) -> Self {
        Self {
            deadline: Instant::now() + duration,
        }
    }
}

impl Future for SpinTimeout {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Duration::ZERO = self.deadline.duration_since(Instant::now()) {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[derive(Clone)]
pub struct ReactorTimeout {
    deadline: Instant,
}

impl ReactorTimeout {
    pub fn new(duration: Duration) -> Self {
        Self {
            deadline: Instant::now() + duration,
        }
    }
}

impl Future for ReactorTimeout {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Duration::ZERO = self.deadline.duration_since(Instant::now()) {
            Poll::Ready(())
        } else {
            REACTOR
                .lock()
                .unwrap()
                .register_timeout(self.deadline, cx.waker().clone());
            Poll::Pending
        }
    }
}

pub trait AsyncRead: Sized {
    fn async_read<'a, 'b>(&'a mut self, buf: &'b mut [u8]) -> Read<'a, 'b, Self>;
}

pub trait AsyncWrite: Sized {
    fn async_write<'a, 'b>(&'a mut self, buf: &'b [u8]) -> Write<'a, 'b, Self>;
}

pub struct Read<'a, 'b, S> {
    source: &'a mut S,
    buf: Option<&'b mut [u8]>,
}

pub struct Write<'a, 'b, S> {
    source: &'a mut S,
    buf: Option<&'b [u8]>,
}

impl<'a, 'b, S> Future for Read<'a, 'b, S>
where
    S: Source + io::Read,
{
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let buf = self.buf.take().unwrap();
        let res = io::Read::read(self.source, buf);
        match res {
            Err(er) if er.kind() == ErrorKind::WouldBlock => {
                let waker = cx.waker().clone();
                REACTOR.lock().unwrap().register_read(self.source, waker);

                self.buf = Some(buf);
                Poll::Pending
            }
            _ => {
                REACTOR.lock().unwrap().deregister(self.source);
                Poll::Ready(res)
            }
        }
    }
}

impl<'a, 'b, S> Future for Write<'a, 'b, S>
where
    S: Source + io::Write,
{
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let buf = self.buf.take().unwrap();
        let res = io::Write::write(self.source, buf);
        match res {
            Err(er) if er.kind() == ErrorKind::WouldBlock => {
                let waker = cx.waker().clone();
                REACTOR.lock().unwrap().register_write(self.source, waker);

                self.buf = Some(buf);
                Poll::Pending
            }
            _ => {
                REACTOR.lock().unwrap().deregister(self.source);
                Poll::Ready(res)
            }
        }
    }
}

impl AsyncRead for TcpStream {
    fn async_read<'a, 'b>(&'a mut self, buf: &'b mut [u8]) -> Read<'a, 'b, Self> {
        Read {
            source: self,
            buf: Some(buf),
        }
    }
}

impl AsyncWrite for TcpStream {
    fn async_write<'a, 'b>(&'a mut self, buf: &'b [u8]) -> Write<'a, 'b, Self> {
        Write {
            source: self,
            buf: Some(buf),
        }
    }
}

pub struct TcpListener {
    inner: mio::net::TcpListener,
}

impl TcpListener {
    pub fn bind(addr: &str) -> io::Result<Self> {
        Ok(Self {
            inner: mio::net::TcpListener::bind(addr.parse().unwrap())?,
        })
    }

    pub fn accept(&self) -> Accept {
        Accept { listener: self }
    }
}

pub struct Accept<'a> {
    listener: &'a TcpListener,
}

impl<'a> Future for Accept<'a> {
    type Output = io::Result<(TcpStream, SocketAddr)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.listener.inner.accept() {
            Err(er) if er.kind() == ErrorKind::WouldBlock => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            other @ _ => Poll::Ready(other),
        }
    }
}
