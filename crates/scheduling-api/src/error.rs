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
}

pub type SchedulerResult<T> = Result<T, SchedulerError>;

impl IntoResponse for SchedulerError {
    fn into_response(self) -> Response {
        let body = match self {
            SchedulerError::Caldav(e) => e.to_string(),
            SchedulerError::Reqwest(err) => err.to_string(),
        };

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
