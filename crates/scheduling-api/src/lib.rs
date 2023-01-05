use axum::{extract::State, http::StatusCode, Json};
use caldav_utils::{calendar::Calendar, principal::Principal};
use serde::{Deserialize, Serialize};
use serde_with::DurationSeconds;
use tracing::info;

pub mod error;
pub mod state;

#[cfg(test)]
mod tests;

pub use crate::{
    error::{SchedulerError, SchedulerResult},
    state::CaldavAvailability,
};

#[serde_with::serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AvailabilityRequest {
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
}

/// The open time slots for a given time range
/// on the calendar that was requested.
/// The slots are determined by finding the gaps between
/// availabile blocks and booked blocks.
#[serde_with::serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AvailabilityResponse {
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
    /// the amount of time to subdivide the availability into.
    /// e.g. if this is 30 minutes, then the availability matrix will contain
    /// a slot for every 30 minutes between start and end.
    #[serde_as(as = "DurationSeconds<i64>")]
    pub granularity: chrono::Duration,
    /// the availability matrix, each value determines whether the availability is
    /// open for the entire granularity period.
    pub matrix: Vec<bool>,
}

pub async fn get_calendar(
    client: &reqwest::Client,
    principal: &mut Principal,
    calendar_name: &str,
) -> SchedulerResult<Calendar> {
    let calendars = principal.get_calendars(client).await?;
    let calendar = calendars
        .iter()
        .find(|c| c.display_name == calendar_name)
        .ok_or_else(|| SchedulerError::CalendarNotFound {
            calendar_name: calendar_name.to_string(),
        })?;
    Ok(calendar.clone())
}

pub async fn get_availability(
    client: &reqwest::Client,
    availability: &Calendar,
    _booked: &Calendar,
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
    granularity: chrono::Duration,
) -> SchedulerResult<AvailabilityResponse> {
    // TODO: lookup events in the calendar
    let events = availability.get_events(&client, start, end).await?;
    info!("found {} events", events.len());

    // Determine the start and end of the availability matrix
    // Assume that the availability matrix always starts with false values (not available)
    let duration = end - start;
    let num_slots = duration.num_minutes() / granularity.num_minutes();
    let matrix = vec![false; num_slots as usize];

    Ok(AvailabilityResponse {
        start,
        end,
        granularity,
        matrix,
    })
}

#[axum::debug_handler]
pub async fn request_availability(
    State(caldav_state): State<CaldavAvailability>,
    body: Json<AvailabilityRequest>,
) -> SchedulerResult<Json<AvailabilityResponse>> {
    // TODO: validate the request. e.g. start < end, max range, etc.

    // First, lookup events in the availability calendar
    let client = reqwest::Client::new();
    let mut principal = caldav_state.davclient.get_principal(&client).await?;
    let availability_calendar =
        get_calendar(&client, &mut principal, &caldav_state.availability_calendar).await?;
    let booked_calendar =
        get_calendar(&client, &mut principal, &caldav_state.booked_calendar).await?;
    info!(
        "Found calendars: {:?}, {:?}",
        availability_calendar, booked_calendar
    );

    let granularity = chrono::Duration::minutes(30);

    let avail = get_availability(
        &client,
        &availability_calendar,
        &booked_calendar,
        body.start,
        body.end,
        granularity,
    )
    .await?;

    // Err(StatusCode::NOT_IMPLEMENTED)
    Ok(Json(avail))
}

/// gets the current time
pub async fn get_now() -> Result<Json<chrono::DateTime<chrono::Utc>>, StatusCode> {
    Ok(Json(chrono::Utc::now()))
}
