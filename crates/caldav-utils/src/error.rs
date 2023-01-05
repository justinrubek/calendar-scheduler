use thiserror::Error;

#[derive(Error, Debug)]
pub enum CaldavError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    InvalidMethod(#[from] http::method::InvalidMethod),
    #[error(transparent)]
    Minidom(#[from] minidom::Error),
}

pub type CaldavResult<T> = Result<T, CaldavError>;
