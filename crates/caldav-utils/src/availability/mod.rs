use chrono::TimeZone;
use icalendar::Component;
use rrule::{RRule, Tz, Unvalidated};
use serde_with::DurationSeconds;
use tracing::info;

use crate::caldav::{calendar::Calendar, event::Event};
use crate::error::CaldavResult;

#[serde_with::serde_as]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AvailabilityRequest {
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
}

/// The open time slots for a given time range
/// on the calendar that was requested.
/// The slots are determined by finding the gaps between
/// availabile blocks and booked blocks.
#[serde_with::serde_as]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
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
    matrix[start_index as usize + 1..end_index as usize + 1]
        .iter_mut()
        .for_each(|x| *x = true);

    matrix
}

pub fn get_event_matrix(
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
    granularity: chrono::Duration,
    event: &Event,
    timezone: Option<String>,
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

    let tz = match timezone {
        Some(tz) => {
            let tz_str = tz.as_str();
            match tz_str {
                "UTC" => Tz::UTC,
                _ => unimplemented!(),
            }
        }
        _ => Tz::UTC,
    };

    // TODO: fix formatting of the date string
    // It may be necessary to add a trailing Z to the date string
    let str_has_z = dtstart_str.ends_with('Z');
    let format = if str_has_z {
        "%Y%m%dT%H%M%SZ"
    } else {
        "%Y%m%dT%H%M%S"
    };

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

    // If there is an RRULE, then we need to generate a list of all the
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
        matrix[begin_index + 1..end_index + 1]
            .iter_mut()
            .for_each(|x| *x = true);
    }

    matrix
}

pub async fn calendar_availability(
    client: &reqwest::Client,
    calendar: &Calendar,
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
    granularity: chrono::Duration,
) -> CaldavResult<Vec<bool>> {
    let duration = end - start;
    let num_slots = duration.num_minutes() / granularity.num_minutes();
    let all_false = vec![false; num_slots as usize];

    Ok(calendar
        .get_events(client, start, end)
        .await?
        .iter()
        .map(|event| get_event_matrix(start, end, granularity, event, calendar.timezone.clone()))
        .fold(all_false, |acc: Vec<bool>, x| {
            acc.iter().zip(x.iter()).map(|(a, b)| *a || *b).collect()
        }))
}

pub async fn get_availability(
    client: &reqwest::Client,
    availability: &Calendar,
    _booked: &Calendar,
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
    granularity: chrono::Duration,
) -> CaldavResult<AvailabilityResponse> {
    let duration = end - start;
    let num_slots = duration.num_minutes() / granularity.num_minutes();

    // lookup events in the calendar
    let events = availability.get_events(client, start, end).await?;
    info!("found {} events", events.len());
    tracing::debug!("events: {:#?}", events);

    let all_false = vec![false; num_slots as usize];
    let matrix = events
        .iter()
        .map(|event| {
            get_event_matrix(
                start,
                end,
                granularity,
                event,
                availability.timezone.clone(),
            )
        })
        .fold(all_false, |acc, x| {
            acc.iter().zip(x.iter()).map(|(a, b)| *a || *b).collect()
        });

    Ok(AvailabilityResponse {
        start,
        end,
        granularity,
        matrix,
    })
}
