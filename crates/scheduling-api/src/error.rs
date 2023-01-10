use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error(transparent)]
    Caldav(#[from] caldav_utils::error::CaldavError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("Requested time not available: {0}")]
    TimeNotAvailable(chrono::DateTime<chrono::Utc>),
}

pub type SchedulerResult<T> = Result<T, SchedulerError>;

impl IntoResponse for SchedulerError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
