use axum::{
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};


#[serde_with::serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Availability {
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
    /// the amount of time to subdivide the availability into.
    /// e.g. if this is 30 minutes, then the availability matrix will contain
    /// a slot for every 30 minutes between start and end.
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    pub granularity: chrono::Duration,
    /// the availability matrix, each value determines whether the availability is
    /// open for the entire granularity period.
    pub matrix: Vec<bool>,
}

pub async fn availability() -> Result<Json<Availability>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}
