use thiserror::Error;

#[derive(Error, Debug)]
pub enum CaldavError {
    #[error("Calendar not found: {calendar_name}")]
    CalendarNotFound { calendar_name: String },
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    InvalidMethod(#[from] http::method::InvalidMethod),
    #[error(transparent)]
    Minidom(#[from] minidom::Error),
}

pub type CaldavResult<T> = Result<T, CaldavError>;
