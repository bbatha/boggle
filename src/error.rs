use std::error;
use std::fmt;
use std::io;
use std::convert;

const USAGE: &str = "USAGE: boggle dictionary board";

#[derive(Debug)]
pub enum Error {
    Usage,
    Io(io::Error),
    BoardSize(&'static str),
}

impl convert::From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        use Error::*;
        match *self {
            Usage => USAGE,
            Io(ref err) => err.description(),
            BoardSize(ref err) => err,
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        use Error::*;
        match *self {
            Usage => None,
            Io(ref err) => Some(err),
            BoardSize(_) => None,
        }
    }
}