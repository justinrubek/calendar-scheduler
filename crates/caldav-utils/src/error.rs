use thiserror::Error;

#[derive(Error, Debug)]
pub enum CaldavError {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    ChronoParse(#[from] chrono::ParseError),
    #[error("Calendar not found: {calendar_name}")]
    CalendarNotFound { calendar_name: String },
    #[error(transparent)]
    InvalidMethod(#[from] http::method::InvalidMethod),
    #[error(transparent)]
    Minidom(#[from] minidom::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    RRule(#[from] rrule::RRuleError),
    #[error("Error calling caldav server: {0}")]
    ServerResponse(String),
}

pub type CaldavResult<T> = Result<T, CaldavError>;
