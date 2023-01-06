use axum::{extract::State, http::StatusCode, Json};
use caldav_utils::{
    availability::{get_availability, AvailabilityRequest, AvailabilityResponse},
    caldav::{calendar::Calendar, principal::Principal},
};
use tracing::info;

pub mod error;
pub mod state;

pub use crate::{
    error::{SchedulerError, SchedulerResult},
    state::CaldavAvailability,
};

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

pub async fn get_calendars(
    client: &reqwest::Client,
    caldav_state: CaldavAvailability,
) -> SchedulerResult<(Calendar, Calendar)> {
    let mut principal = caldav_state.davclient.get_principal(client).await?;

    Ok((
        get_calendar(client, &mut principal, &caldav_state.availability_calendar).await?,
        get_calendar(client, &mut principal, &caldav_state.booked_calendar).await?,
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
