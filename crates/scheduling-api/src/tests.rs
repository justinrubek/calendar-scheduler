use caldav_utils::{caldav::event::Event, format::DATETIME};
use icalendar::{Component, Property};

use crate::get_event_matrix;

fn build_event(
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
    rrule: Option<String>,
) -> icalendar::Event {
    let mut event = icalendar::Event::new();

    // because calendar timezones are not supported, we need to convert the start and end times to
    // US/Central time as the library assumes that is the timezone of the calendar.
    let timezone = chrono_tz::US::Central;
    let start = start.with_timezone(&timezone);
    let end = end.with_timezone(&timezone);

    let start_str = format!("{}", start.format(DATETIME));
    let end_str = format!("{}", end.format(DATETIME));

    event.append_property(Property::new("DTSTART", &start_str));
    event.append_property(Property::new("DTEND", &end_str));
    if let Some(rrule) = rrule {
        event.append_property(Property::new("RRULE", &rrule));
    }

    event
}

fn build_calendar(events: Vec<icalendar::Event>) -> icalendar::Calendar {
    let mut calendar = icalendar::Calendar::new();
    events.into_iter().for_each(|event| {
        calendar.push(event);
    });

    calendar
}

// Macro for generating a test case.
// Should accept a start time, an end time, any RRULE, granularity, and the
// expected matrix.
fn build_matrix_test(
    range_start: chrono::DateTime<chrono::Utc>,
    range_end: chrono::DateTime<chrono::Utc>,
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
    rrule: Option<String>,
    granularity: chrono::Duration,
) -> Vec<bool> {
    let events = vec![build_event(start, end, rrule)];
    let calendar = build_calendar(events);
    let event = Event::new(calendar);

    let matrix = get_event_matrix(range_start, range_end, granularity, &event);

    matrix
}

fn print_matrix_diff(left: &Vec<bool>, right: &Vec<bool>) {
    let mut left_iter = left.iter();
    let mut right_iter = right.iter();
    let mut index = 0;
    loop {
        let left = left_iter.next();
        let right = right_iter.next();
        if left.is_none() && right.is_none() {
            break;
        }
        if left != right {
            println!("index: {index}, left: {left:?}, right: {right:?}");
        }
        index += 1;
    }
}

#[tokio::test]
async fn availability_30_min() {
    let range_start = chrono::Utc::now();
    let range_end = range_start + chrono::Duration::days(1);
    let granularity = chrono::Duration::minutes(30);

    let event_start = range_start + chrono::Duration::hours(1);
    let event_end = event_start + chrono::Duration::hours(1);
    let rrule: Option<String> = None;

    let mut expected = vec![false; 48];
    expected[2..4].iter_mut().for_each(|x| *x = true);

    // Test case 1: 30 minute event, no RRULE.
    // build_matrix_test!(start, end, rrule, granularity, expected);
    // no macro
    let res = build_matrix_test(
        range_start,
        range_end,
        event_start,
        event_end,
        rrule,
        granularity,
    );
    println!("expected: {:?}", expected);
    println!("res: {:?}", res);
    assert_eq!(res, expected);

    assert!(true);
}

#[tokio::test]
async fn availability_30_min_rrule() {
    let range_start = chrono::Utc::now();
    let range_end = range_start + chrono::Duration::days(5);
    let granularity = chrono::Duration::minutes(30);

    let event_start = range_start + chrono::Duration::hours(1);
    let event_end = event_start + chrono::Duration::hours(1);
    let rrule = Some("FREQ=DAILY;COUNT=2".to_string());

    let mut expected = vec![false; 240];
    expected[2..4].iter_mut().for_each(|x| *x = true);
    expected[50..52].iter_mut().for_each(|x| *x = true);

    // Test case 2: 30 minute event, RRULE.
    // build_matrix_test!(start, end, rrule, granularity, expected);
    // no macro
    let res = build_matrix_test(
        range_start,
        range_end,
        event_start,
        event_end,
        rrule,
        granularity,
    );
    print_matrix_diff(&res, &expected);
    assert_eq!(res, expected);

    assert!(true);
}
