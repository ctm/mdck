use std::error::Error;
use std::fmt;
use structopt::clap;

#[derive(Debug)]
pub enum MdckError {
    Internal(String),
    Io(std::io::Error),
    FromUtf(std::string::FromUtf8Error),
    Clap(clap::Error),
}

impl fmt::Display for MdckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            MdckError::Internal(message) => message,
            MdckError::Io(err) => err.description(),
            MdckError::FromUtf(err) => err.description(),
            MdckError::Clap(err) => err.description(),
        };
        write!(f, "{}", message)
    }
}

impl Error for MdckError {}

impl From<&str> for MdckError {
    fn from(error: &str) -> Self {
        MdckError::Internal(error.to_string())
    }
}

impl From<std::io::Error> for MdckError {
    fn from(error: std::io::Error) -> Self {
        MdckError::Io(error)
    }
}

impl From<std::string::FromUtf8Error> for MdckError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        MdckError::FromUtf(error)
    }
}

impl From<clap::Error> for MdckError {
    fn from(error: clap::Error) -> Self {
        MdckError::Clap(error)
    }
}
