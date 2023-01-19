use anyhow::anyhow;
use chrono::TimeZone;
use icalendar::Component;
use rrule::{RRule, Tz, Unvalidated};
use serde_with::DurationSeconds;
use tracing::info;

use crate::caldav::{calendar::Calendar, event::Event};
use crate::error::{CaldavError, CaldavResult};

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

/// Parse a datetime string from icalendar
/// This may or may not have the timezone specified.
/// TODO: support timezones other than UTC?
fn parse_datetime_from_str(tz: &Tz, datetime_str: &str) -> CaldavResult<chrono::DateTime<Tz>> {
    if datetime_str.ends_with('Z') {
        let datetime = tz.datetime_from_str(datetime_str, "%Y%m%dT%H%M%SZ")?;
        Ok(datetime)
    } else {
        let datetime = tz.datetime_from_str(datetime_str, "%Y%m%dT%H%M%S")?;
        Ok(datetime)
    }
}

pub fn get_num_slots(
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
    granularity: chrono::Duration,
) -> usize {
    let duration = end - start;
    (duration.num_minutes() / granularity.num_minutes()) as usize
}

// generates a matrix for an event with no RRULE
pub fn generate_matrix_no_rrule(
    // the start of the range that we're generating the matrix for
    range_start: chrono::DateTime<chrono::Utc>,
    // the start of the availability event - times in this range are considered available
    event_start: chrono::DateTime<chrono::Utc>,
    event_end: chrono::DateTime<chrono::Utc>,
    num_slots: i64,
    granularity: chrono::Duration,
) -> CaldavResult<Vec<bool>> {
    let mut matrix = vec![false; num_slots as usize];

    tracing::debug!(
        r#"generating matrix for event with no RRULE,
            range_start: {range_start}, 
            event_start: {event_start},
            event_end: {event_end},
            num_slots: {num_slots},
            granularity: {granularity}"#,
    );

    let start_index = {
        if event_start == range_start {
            0
        } else {
            let diff = event_start - range_start;
            let index = diff.num_minutes() / granularity.num_minutes();
            if index < 0 {
                0
            } else if index >= num_slots {
                return Ok(matrix);
            } else {
                index + 1
            }
        }
    } as usize;
    let end_index = {
        if event_end == range_start {
            0
        } else {
            let diff = event_end - range_start;
            let index = diff.num_minutes() / granularity.num_minutes();
            tracing::debug!("diff: {:#?}", diff);
            tracing::debug!("end_index: {}", index);
            if index + 1 > num_slots {
                num_slots
            } else {
                index + 1
            }
        }
    } as usize;
    tracing::debug!("start_index: {start_index}, end_index: {end_index}",);

    matrix[start_index..end_index]
        .iter_mut()
        .for_each(|x| *x = true);
    tracing::debug!("matrix: {:#?}", matrix);
    Ok(matrix)
}

fn get_rruleset(event: &icalendar::Event, tz: &Tz) -> CaldavResult<Option<rrule::RRuleSet>> {
    let rrule_str = match event.property_value("RRULE") {
        None => return Ok(None),
        Some(rule) => rule,
    };

    let dtstart = {
        let as_str = event
            .property_value("DTSTART")
            .ok_or_else(|| CaldavError::Anyhow(anyhow!("DTSTART not found")))?;
        let dtstart_local = parse_datetime_from_str(tz, as_str)?;
        Tz::UTC.from_utc_datetime(&dtstart_local.naive_utc())
    };

    let rrule: RRule<Unvalidated> = rrule_str.parse()?;
    let rrule = rrule.build(dtstart)?;
    Ok(Some(rrule))
}

pub fn get_event_matrix(
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
    granularity: chrono::Duration,
    event: &Event,
    timezone: Option<String>,
) -> CaldavResult<Vec<bool>> {
    if end < start {
        tracing::warn!("end is before start");
        return Ok(vec![]);
    }

    let num_slots = (end - start).num_minutes() / granularity.num_minutes();

    tracing::debug!("generating matrix for event: {:#?}", event);
    // determine the time of the event compared to the requested time range.
    // First, we need to get the properties from the inner icalendar::Event.
    let comps = &event.ical.components;
    // Assume there is only one component.
    let event_comp = comps
        .iter()
        .find(|c| matches!(c, icalendar::CalendarComponent::Event(_)));
    if event_comp.is_none() {
        return Err(CaldavError::Anyhow(anyhow!(
            "no event component found in event"
        )));
    }
    let event = event_comp.unwrap().as_event().unwrap();

    // Get the start and end times of the event.
    let dtstart_str = event.property_value("DTSTART").unwrap();
    let dtend_str = event.property_value("DTEND").unwrap();
    tracing::debug!("dtstart_str: {:#?}", dtstart_str);
    tracing::debug!("dtend_str: {:#?}", dtend_str);

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

    match event.property_value("RRULE") {
        Some(_) => generate_matrix_rrule(event, &tz, start, end, num_slots, granularity),
        None => {
            let dtstart_local = tz.datetime_from_str(dtstart_str, format).unwrap();
            let dtend_local = tz.datetime_from_str(dtend_str, format).unwrap();
            // Convert the start and end times to UTC.
            let dtstart = chrono::Utc
                .from_local_datetime(&dtstart_local.naive_utc())
                .unwrap();
            let dtend = chrono::Utc
                .from_local_datetime(&dtend_local.naive_utc())
                .unwrap();
            tracing::debug!("dtstart_local: {:#?}", dtstart_local);
            tracing::debug!("dtend_local: {:#?}", dtend_local);
            tracing::debug!("dtstart: {:#?}", dtstart);
            tracing::debug!("dtend: {:#?}", dtend);
            generate_matrix_no_rrule(start, dtstart, dtend, num_slots, granularity)
        }
    }
}

pub async fn calendar_availability(
    client: &reqwest::Client,
    calendar: &Calendar,
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
    granularity: chrono::Duration,
) -> CaldavResult<Vec<bool>> {
    let num_slots = get_num_slots(start, end, granularity);
    let all_false = vec![false; num_slots];

    Ok(calendar
        .get_events(client, start, end)
        .await?
        .iter()
        .map(|event| get_event_matrix(start, end, granularity, event, calendar.timezone.clone()))
        .fold(all_false, |acc: Vec<bool>, x| {
            let x = x.unwrap();
            acc.iter().zip(x.iter()).map(|(a, b)| *a || *b).collect()
        }))
}

pub async fn get_availability(
    client: &reqwest::Client,
    availability: &Calendar,
    booked: &Calendar,
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
            let x = x.unwrap();
            acc.iter().zip(x.iter()).map(|(a, b)| *a || *b).collect()
        });

    // Now, we need to do the same thing for the booked calendar, but we need to
    // invert the matrix modifications so that the booked times are marked as unavailable.
    let booked_events = booked.get_events(client, start, end).await?;
    info!("found {} booked events", booked_events.len());
    tracing::debug!("booked_events: {:#?}", booked_events);

    let booked_matrix = booked_events
        .iter()
        .map(|event| get_event_matrix(start, end, granularity, event, booked.timezone.clone()))
        .fold(matrix, |acc, x| {
            let x = x.unwrap();
            acc.iter().zip(x.iter()).map(|(a, b)| *a && !b).collect()
        });

    Ok(AvailabilityResponse {
        start,
        end,
        granularity,
        matrix: booked_matrix,
    })
}

pub fn generate_matrix_rrule(
    // event containing availability
    event: &icalendar::Event,
    tz: &Tz,
    // start of the availability matrix
    start: chrono::DateTime<chrono::Utc>,
    // end of the availability matrix
    end: chrono::DateTime<chrono::Utc>,
    num_slots: i64,
    granularity: chrono::Duration,
) -> CaldavResult<Vec<bool>> {
    let dtstart = {
        let dtstart_str = event.property_value("DTSTART").unwrap();
        tracing::debug!("dtstart: {:#?}", dtstart_str);
        let dtstart_local = parse_datetime_from_str(tz, dtstart_str)?;
        chrono::Utc
            .from_local_datetime(&dtstart_local.naive_utc())
            .unwrap()
    };
    let dtend = {
        let dtend_str = event.property_value("DTEND").unwrap();
        let dtend_local = parse_datetime_from_str(tz, dtend_str)?;
        chrono::Utc
            .from_local_datetime(&dtend_local.naive_utc())
            .unwrap()
    };

    let tz_start = Tz::UTC.from_utc_datetime(&dtstart.naive_utc());

    // Convert the requested time-range to rrule compatible datetimes
    let range_tz_end = Tz::UTC.from_utc_datetime(&end.naive_utc());

    let rrule = get_rruleset(event, tz)?.unwrap();

    let (detected_events, _) = rrule.after(tz_start).before(range_tz_end).all(100);
    tracing::debug!("detected_events: {:#?}", detected_events);

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
    tracing::debug!("event_ranges: {:#?}", event_ranges);

    // Now that the rrule has been resolved to multiple events, we can treat them
    // the same as events without an rrule
    let final_matrix = event_ranges
        .iter()
        .filter_map(|(begin, end)| {
            // only include events that are within the requested time range
            if begin < end && end > &start {
                Some(generate_matrix_no_rrule(
                    start,
                    *begin,
                    *end,
                    num_slots,
                    granularity,
                ))
            } else {
                None
            }
        })
        .fold(vec![false; num_slots as usize], |acc, x| {
            let x = x.unwrap();
            acc.iter().zip(x.iter()).map(|(a, b)| *a || *b).collect()
        });

    Ok(final_matrix)
}
