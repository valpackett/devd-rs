extern crate nix;
#[macro_use]
extern crate nom;

pub mod result;
pub mod data;
pub mod parser;

use nix::sys::socket::*;
use nix::sys::event::*;
use nix::unistd::close;
use std::os::unix::io::{RawFd, FromRawFd};
use std::os::unix::net::UnixStream;
use std::io;
use io::{BufRead, BufReader};

pub use result::*;
pub use data::*;

const SOCKET_PATH: &'static str = "/var/run/devd.seqpacket.pipe";

#[derive(Debug)]
pub struct Context {
    sock: BufReader<UnixStream>,
    kq: RawFd,
}

impl Drop for Context {
    fn drop(&mut self) {
        close(self.kq);
    }
}

impl Context {
    pub fn new() -> Result<Context> {
        let sockfd = socket(AddressFamily::Unix, SockType::SeqPacket, SockFlag::empty(), 0)?;
        connect(sockfd, &SockAddr::Unix(UnixAddr::new(SOCKET_PATH)?))?;
        let kq = kqueue()?;
        kevent(kq, &vec![
               KEvent::new(sockfd as usize, EventFilter::EVFILT_READ, EV_ADD | EV_ENABLE, FilterFlag::empty(), 0, 0)
        ], &mut vec![], 0)?;
        Ok(Context {
            sock: BufReader::new(unsafe { UnixStream::from_raw_fd(sockfd) }),
            kq: kq
        })
    }

    pub fn wait_for_event_raw(&mut self) -> Result<String> {
        let mut eventlist = vec![KEvent::new(0, EventFilter::EVFILT_READ, EventFlag::empty(), FilterFlag::empty(), 0, 0)];
        kevent_ts(self.kq, &vec![], &mut eventlist, None)?;
        let mut s = String::new();
        self.sock.read_line(&mut s);
        Ok(s)
    }

    pub fn wait_for_event<'a>(&mut self) -> Result<Event> {
        self.wait_for_event_raw().and_then(|e| {
            match parser::event(e.as_bytes()) {
                parser::IResult::Done(_, x) => Ok(x),
                _ => Err(Error::from(io::Error::new(io::ErrorKind::Other, "devd parse error")))
            }
        })
    }
}
