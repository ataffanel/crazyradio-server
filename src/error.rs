use core::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    DeserializeError(serde_json::Error),
    CrazyradioError(crazyradio::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::DeserializeError(error)
    }
}

impl From<crazyradio::Error> for Error {
    fn from(error: crazyradio::Error) -> Self {
        Error::CrazyradioError(error)
    }
}
