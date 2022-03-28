use config::{ConfigError};
use tokio::task::JoinError;
use hyper::http::{Error as HttpError};
use hyper::http::uri::InvalidUri;
use hyper::http::header::{InvalidHeaderValue, ToStrError};
use hyper::{Error};

#[derive(Debug)]
pub enum AppError {
    ConfigError(String),
    JoinError(JoinError),
    IOError(std::io::Error),
    UriError(String),
    InvalidUri(InvalidUri),
    HttpError(HttpError),
    HyperError(Error),
    RuntimeError(String),
    ForwardHeaderError,
}

impl std::error::Error for AppError {}

impl std::fmt::Display for AppError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{:?}", self)
    }
}

impl From<ConfigError> for AppError {
    fn from(err: ConfigError) -> Self {
        AppError::ConfigError(format!("{}", err))
    }
}

impl From<JoinError> for AppError {
    fn from(err: JoinError) -> Self {
        AppError::JoinError(err)
    }
}


impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IOError(err)
    }
}

impl From<InvalidUri> for AppError {
    fn from(err: InvalidUri) -> Self {
        AppError::InvalidUri(err)
    }
}

impl From<HttpError> for AppError {
    fn from(err: HttpError) -> Self {
        AppError::HttpError(err)
    }
}

impl From<Error> for AppError {
    fn from(err: Error) -> Self {
        AppError::HyperError(err)
    }
}

impl From<ToStrError> for AppError {
    fn from(_err: ToStrError) -> Self {
        AppError::ForwardHeaderError
    }
}

impl From<InvalidHeaderValue> for AppError {
    fn from(_err: InvalidHeaderValue) -> Self {
        AppError::ForwardHeaderError
    }
}
