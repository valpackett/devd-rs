extern crate nix;
extern crate libc;
#[macro_use]
extern crate nom;

pub mod result;
pub mod data;
pub mod parser;

use nix::sys::socket::*;
use libc::{nfds_t, c_int, poll, pollfd, POLLIN};
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
    sockfd: RawFd,
}

impl Context {
    pub fn new() -> Result<Context> {
        let sockfd = socket(AddressFamily::Unix, SockType::SeqPacket, SockFlag::empty(), 0)?;
        connect(sockfd, &SockAddr::Unix(UnixAddr::new(SOCKET_PATH)?))?;
        Ok(Context {
            sock: BufReader::new(unsafe { UnixStream::from_raw_fd(sockfd) }),
            sockfd: sockfd,
        })
    }

    pub fn wait_for_event_raw(&mut self, timeout_ms: usize) -> Result<String> {
        let mut fds = vec![
            pollfd {
                fd: self.sockfd,
                events: POLLIN,
                revents: 0
            }
        ];
        let x = unsafe { poll((&mut fds).as_mut_ptr(), fds.len() as nfds_t, timeout_ms as c_int) };
        if x == 0 {
            Err(Error::from(io::Error::new(io::ErrorKind::Other, "timeout")))
        } else {
            let mut s = String::new();
            let _ = self.sock.read_line(&mut s);
            Ok(s)
        }
    }

    pub fn wait_for_event<'a>(&mut self, timeout_ms: usize) -> Result<Event> {
        self.wait_for_event_raw(timeout_ms).and_then(|e| {
            match parser::event(e.as_bytes()) {
                parser::IResult::Done(_, x) => Ok(x),
                _ => Err(Error::from(io::Error::new(io::ErrorKind::Other, "devd parse error")))
            }
        })
    }
}
