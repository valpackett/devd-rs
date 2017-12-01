use nix;
use std::{io, result};

#[derive(Debug)]
pub struct Error(io::Error);

impl Into<io::Error> for Error {
    fn into(self) -> io::Error {
        match self {
            Error(e) => e,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error(err)
    }
}

impl From<nix::Error> for Error {
    fn from(err: nix::Error) -> Error {
        Error(match err {
            nix::Error::Sys(errno) => io::Error::from_raw_os_error(errno as i32),
            nix::Error::InvalidPath => io::Error::new(io::ErrorKind::InvalidInput, "Invalid path"),
            nix::Error::InvalidUtf8 => io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"),
            nix::Error::UnsupportedOperation => io::Error::new(io::ErrorKind::Other, "Unsupported operation"),
        })
    }
}

pub type Result<T> = result::Result<T, Error>;
