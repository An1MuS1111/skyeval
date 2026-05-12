#[derive(Debug, PartialEq)]
pub enum SkyeError {}

impl fmt::Display for SkyeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {}
    }
}

pub type SkyeResult<T> = Result<T, SkyeError>;
