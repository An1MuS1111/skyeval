use std::fmt;

#[derive(Debug, PartialEq)]
pub enum SkyeError {
    Unknown,
}

impl fmt::Display for SkyeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SkyeError::Unknown => write!(f, "Unknown error"),
        }
    }
}

pub type SkyeResult<T> = Result<T, SkyeError>;
