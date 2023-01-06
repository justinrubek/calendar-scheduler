use axum::{extract::State, http::StatusCode, Json};
use caldav_utils::{
    availability::{get_availability, AvailabilityRequest, AvailabilityResponse},
    caldav::calendar::Calendar,
};
use tracing::info;

pub mod error;
pub mod state;

pub use crate::{
    error::{SchedulerError, SchedulerResult},
    state::CaldavAvailability,
};

pub async fn get_calendars(
    client: &reqwest::Client,
    caldav_state: CaldavAvailability,
) -> SchedulerResult<(Calendar, Calendar)> {
    let mut principal = caldav_state.davclient.get_principal(client).await?;

    Ok((
        principal
            .get_calendar(client, &caldav_state.availability_calendar)
            .await?,
        principal
            .get_calendar(client, &caldav_state.booked_calendar)
            .await?,
    ))
}

#[axum::debug_handler]
pub async fn request_availability(
    State(caldav_state): State<CaldavAvailability>,
    body: Json<AvailabilityRequest>,
) -> SchedulerResult<Json<AvailabilityResponse>> {
    // TODO: validate the request. e.g. start < end, max range, etc.

    // First, lookup events in the availability calendar
    let client = reqwest::Client::new();
    let (availability_calendar, booked_calendar) = get_calendars(&client, caldav_state).await?;
    info!(
        "Found calendars: {}, {}",
        availability_calendar.path, booked_calendar.path
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
