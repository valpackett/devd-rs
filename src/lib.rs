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

pub fn parse_devd_event(e: String) -> Result<Event> {
    match parser::event(e.as_bytes()) {
        parser::IResult::Done(_, x) => Ok(x),
        _ => Err(Error::from(io::Error::new(io::ErrorKind::Other, "devd parse error")))
    }
}

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

    /// Waits for an event using poll(), reads it but does not parse
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

    /// Waits for an event using poll(), reads and parses it
    pub fn wait_for_event<'a>(&mut self, timeout_ms: usize) -> Result<Event> {
        self.wait_for_event_raw(timeout_ms).and_then(parse_devd_event)
    }

    /// Returns the devd socket file descriptor in case you want to select/poll on it together with
    /// other file descriptors
    pub fn fd(&self) -> RawFd {
        self.sockfd
    }

    /// Reads an event and parses it. Use when polling on the raw fd by yourself
    pub fn read_event(&mut self) -> Result<Event> {
        let mut s = String::new();
        let _ = self.sock.read_line(&mut s);
        parse_devd_event(s)
    }

}
