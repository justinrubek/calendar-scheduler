use axum::{extract::State, http::StatusCode, Json};
use caldav_utils::{calendar::Calendar, principal::Principal};
use chrono::TimeZone;
use icalendar::Component;
use rrule::{RRule, Tz, Unvalidated};
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

// generates a matrix for an event with no RRULE
pub fn generate_matrix_no_rrule(
    range_start: chrono::DateTime<chrono::Utc>,
    event_start: chrono::DateTime<chrono::Utc>,
    event_end: chrono::DateTime<chrono::Utc>,
    num_slots: i64,
    granularity: chrono::Duration,
) -> Vec<bool> {
    let mut matrix = vec![false; num_slots as usize];

    println!(
        r#"generating matrix for event with no RRULE,
            range_start: {range_start}, 
            event_start: {event_start},
            event_end: {event_end},
            num_slots: {num_slots},
            granularity: {granularity}"#,
    );
    
    // modify the matrix to reflect the events
    let start_index = (event_start - range_start).num_minutes() / granularity.num_minutes();
    // the end index should be either the end of the event or the end of the range, whichever is earlier
    let end_index = std::cmp::min(
        (event_end - range_start).num_minutes() / granularity.num_minutes(),
        num_slots,
    );
    println!("start_index: {start_index}, end_index: {end_index}",);
    matrix[start_index as usize..end_index as usize].iter_mut().for_each(|x| *x = true);

    matrix
}

pub fn get_event_matrix(
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
    granularity: chrono::Duration,
    event: &caldav_utils::event::Event,
) -> Vec<bool> {
    let num_slots = (end - start).num_minutes() / granularity.num_minutes();

    info!("generating matrix for event: {:#?}", event);
    // determine the time of the event compared to the requested time range.
    // First, we need to get the properties from the inner icalendar::Event.
    let comps = &event.ical.components;
    // Assume there is only one component.
    let comp = comps.first().unwrap();
    let event = comp.as_event().unwrap();
    info!("component: {:#?}", comp);

    // Get the start and end times of the event.
    let dtstart_str = event.property_value("DTSTART").unwrap();
    let dtend_str = event.property_value("DTEND").unwrap();
    info!("dtstart_str: {:#?}", dtstart_str);
    info!("dtend_str: {:#?}", dtend_str);

    // TODO: Determine the timezone of the calendar
    // For now, assume the time comes from US/Central
    let tz = Tz::US__Central;
    
    // TODO: fix formatting of the date string
    // It may be necessary to add a trailing Z to the date string
    let str_has_z = dtstart_str.ends_with('Z');
    let format = if str_has_z {
        "%Y%m%dT%H%M%SZ"
    } else {
        "%Y%m%dT%H%M%S"
    };

    // rrule uses its own timezone enum, so we need to convert
    // through it to get the chrono::DateTime
    let dtstart_local = tz.datetime_from_str(dtstart_str, format).unwrap();
    let dtend_local = tz.datetime_from_str(dtend_str, format).unwrap();
    // Convert the start and end times to UTC.
    let dtstart = chrono::Utc
        .from_local_datetime(&dtstart_local.naive_utc())
        .unwrap();
    let dtend = chrono::Utc
        .from_local_datetime(&dtend_local.naive_utc())
        .unwrap();
    info!("dtstart_local: {:#?}", dtstart_local);
    info!("dtend_local: {:#?}", dtend_local);
    info!("dtstart: {:#?}", dtstart);
    info!("dtend: {:#?}", dtend);
    let tz_start = Tz::UTC.from_utc_datetime(&dtstart.naive_utc());

    let range_tz_start = Tz::UTC.from_utc_datetime(&start.naive_utc());
    let range_tz_end = Tz::UTC.from_utc_datetime(&end.naive_utc());

    // TODO: If there is an RRULE, then we need to generate a list of all the
    // times that the event occurs. If there is no RRULE, then we can just
    // use the start and end times.
    let rrule_str = match event.property_value("RRULE") {
        None => return generate_matrix_no_rrule(start, dtstart, dtend, num_slots, granularity),
        Some(rule) => rule,
    };

    info!("rrule_str: {:#?}", rrule_str);


    // Determine the recurrence rule of the event.
    let rrule: RRule<Unvalidated> = rrule_str.parse().unwrap();
    let rrule = rrule.build(tz_start).unwrap();
    info!("rrule: {:#?}", rrule);
    let (detected_events, _) = rrule.after(range_tz_start).before(range_tz_end).all(100);
    info!("detected_events: {:#?}", detected_events);

    // for each event, determine the time range it covers.
    let event_duration = dtend - dtstart;
    let event_ranges = detected_events
        .iter()
        .map(|e| {
            let start = chrono::Utc.from_local_datetime(&e.naive_utc()).unwrap();
            let end = start + event_duration;
            (start, end)
        })
        .collect::<Vec<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>>();
    info!("event_ranges: {:#?}", event_ranges);

    let mut matrix = vec![false; num_slots as usize];
    // open slots in the matrix according to the event_ranges
    // this should be done by comparing the start of the event to the
    // start of the availability matrix, finding the index of slots
    // that occur during the event, and setting the values to `true`
    for (begin, end) in event_ranges {
        let begin_index = ((begin - start).num_minutes() / granularity.num_minutes()) as usize;
        let end_index = ((end - start).num_minutes() / granularity.num_minutes()) as usize;
        info!("begin_index: {:#?}", begin_index);
        info!("end_index: {:#?}", end_index);
        matrix[begin_index..end_index]
            .iter_mut()
            .for_each(|x| *x = true);
    }

    matrix
}

pub async fn get_availability(
    client: &reqwest::Client,
    availability: &Calendar,
    _booked: &Calendar,
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
    granularity: chrono::Duration,
) -> SchedulerResult<AvailabilityResponse> {
    let duration = end - start;
    let num_slots = duration.num_minutes() / granularity.num_minutes();

    // lookup events in the calendar
    let events = availability.get_events(client, start, end).await?;
    info!("found {} events", events.len());
    tracing::debug!("events: {:#?}", events);

    // TODO: Support multiple events. For now, assume only the first event is relevant.
    // If there are no events, then the entire time range is unavailable.
    // Do not return an error, just return matrix of false.
    let event = events.first();
    let matrix = match event {
        Some(event) => get_event_matrix(start, end, granularity, event),
        // If there are no events, then there is no availability.
        None => vec![false; num_slots as usize],
    };

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
