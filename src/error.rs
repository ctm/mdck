use std::{borrow::Cow, error::Error, fmt};
use structopt::clap;

#[derive(Debug)]
pub enum MdckError {
    Internal(String),
    Io(std::io::Error),
    FromUtf(std::string::FromUtf8Error),
    Clap(clap::Error),
    WalkDir(walkdir::Error),
}

impl fmt::Display for MdckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut walkdir_err;

        let message = match self {
            MdckError::Internal(message) => message,
            MdckError::Io(err) => err.description(),
            MdckError::FromUtf(err) => err.description(),
            MdckError::Clap(err) => err.description(),
            MdckError::WalkDir(err) => {
                let path = err
                    .path()
                    .map_or(Cow::from("WalkDir failed"), |p| p.to_string_lossy());

                walkdir_err = format!("{}: {}", path, err.description());
                &walkdir_err
            }
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

impl From<walkdir::Error> for MdckError {
    fn from(error: walkdir::Error) -> Self {
        MdckError::WalkDir(error)
    }
}
