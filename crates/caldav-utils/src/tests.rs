use icalendar::{Component, EventLike, Property};

use crate::{
    availability::{generate_matrix_no_rrule, get_event_matrix, get_num_slots},
    caldav::event::Event,
    format::DATETIME,
};

fn build_event(
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
    rrule: Option<String>,
) -> icalendar::Event {
    let mut event = icalendar::Event::new();

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

    let matrix = get_event_matrix(range_start, range_end, granularity, &event, None);

    matrix.expect("Failed to build matrix")
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

#[tokio::test]
async fn test_within_event() -> Result<(), Box<dyn std::error::Error>> {
    // Availability event details
    let dtstart = "20230112T130000";
    let dtend = "20230112T160000";
    let rrule = "FREQ=WEEKLY;COUNT=5";

    // request details
    let start = chrono::DateTime::parse_from_rfc3339("2023-01-12T14:00:00.000000000Z").unwrap();
    let end = chrono::DateTime::parse_from_rfc3339("2023-01-12T14:30:00.000000000Z").unwrap();
    let start = start.with_timezone(&chrono::Utc);
    let end = end.with_timezone(&chrono::Utc);

    let mut event = icalendar::Event::new();
    event.append_property(Property::new("DTSTART", dtstart));
    event.append_property(Property::new("DTEND", dtend));
    event.append_property(Property::new("RRULE", rrule));

    let mut calendar = icalendar::Calendar::new();
    calendar.push(event);

    let event = Event::new(calendar);

    let matrix = get_event_matrix(start, end, chrono::Duration::minutes(30), &event, None)?;
    assert_eq!(matrix, vec![true]);

    Ok(())
}

#[tokio::test]
async fn tests_no_rrule() -> Result<(), Box<dyn std::error::Error>> {
    // Basic event, only support for start and end times.
    struct AvailabilityRange {
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    };

    // Attempts to view the availability from the given start time to the given end time.
    struct TestCase {
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
        expected: Vec<bool>,
    };

    fn print_test_details(test_case: &TestCase, matrix: &Vec<bool>, num_slots: usize) {
        println!(
            r#"
            Test case:
                start: {}
                end: {}
                num_slots: {}
                expected: {:?}
                actual: {:?}
            "#,
            test_case.start, test_case.end, num_slots, test_case.expected, matrix,
        );
    }

    let availability_ranges = vec![
        AvailabilityRange {
            start: chrono::DateTime::parse_from_rfc3339("2023-01-12T14:00:00.000000000Z")
                .unwrap()
                .into(),
            end: chrono::DateTime::parse_from_rfc3339("2023-01-12T16:30:00.000000000Z")
                .unwrap()
                .into(),
        },
        AvailabilityRange {
            start: chrono::DateTime::parse_from_rfc3339("2023-01-13T14:00:00.000000000Z")
                .unwrap()
                .into(),
            end: chrono::DateTime::parse_from_rfc3339("2023-01-13T16:30:00.000000000Z")
                .unwrap()
                .into(),
        },
        AvailabilityRange {
            start: chrono::DateTime::parse_from_rfc3339("2023-01-14T14:00:00.000000000Z")
                .unwrap()
                .into(),
            end: chrono::DateTime::parse_from_rfc3339("2023-01-14T16:30:00.000000000Z")
                .unwrap()
                .into(),
        },
    ];

    let test_cases = vec![
        TestCase {
            start: chrono::DateTime::parse_from_rfc3339("2023-01-12T14:00:00.000000000Z")
                .unwrap()
                .into(),
            end: chrono::DateTime::parse_from_rfc3339("2023-01-12T14:30:00.000000000Z")
                .unwrap()
                .into(),
            expected: vec![true],
        },
        TestCase {
            start: chrono::DateTime::parse_from_rfc3339("2023-01-12T14:30:00.000000000Z")
                .unwrap()
                .into(),
            end: chrono::DateTime::parse_from_rfc3339("2023-01-12T15:00:00.000000000Z")
                .unwrap()
                .into(),
            expected: vec![true],
        },
        TestCase {
            start: chrono::DateTime::parse_from_rfc3339("2023-01-12T16:00:00.000000000Z")
                .unwrap()
                .into(),
            end: chrono::DateTime::parse_from_rfc3339("2023-01-12T16:30:00.000000000Z")
                .unwrap()
                .into(),
            expected: vec![true],
        },
        TestCase {
            start: chrono::DateTime::parse_from_rfc3339("2023-01-12T16:30:00.000000000Z")
                .unwrap()
                .into(),
            end: chrono::DateTime::parse_from_rfc3339("2023-01-12T17:00:00.000000000Z")
                .unwrap()
                .into(),
            expected: vec![false],
        },
        TestCase {
            start: chrono::DateTime::parse_from_rfc3339("2023-01-12T17:00:00.000000000Z")
                .unwrap()
                .into(),
            end: chrono::DateTime::parse_from_rfc3339("2023-01-12T17:30:00.000000000Z")
                .unwrap()
                .into(),
            expected: vec![false],
        },
    ];

    for test_case in test_cases {
        let num_slots = get_num_slots(
            test_case.start,
            test_case.end,
            chrono::Duration::minutes(30),
        );
        println!("num_slots: {}", num_slots);
        // Iterate over availability, and combine the results into a single matrix of the same size.
        let matrix = availability_ranges
            .iter()
            .map(|event| {
                generate_matrix_no_rrule(
                    test_case.start,
                    event.start,
                    event.end,
                    num_slots as i64,
                    chrono::Duration::minutes(30),
                )
            })
            .fold(vec![false; num_slots], |acc, matrix| {
                let matrix = matrix.unwrap();
                acc.iter()
                    .zip(matrix.iter())
                    .map(|(a, b)| *a || *b)
                    .collect()
            });

        print_test_details(&test_case, &matrix, num_slots);
        assert_eq!(matrix, test_case.expected);
    }

    Ok(())
}
