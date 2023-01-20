use icalendar::{Component, EventLike};

pub fn create_event(
    id: &str,
    name: &str,
    description: &str,
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
) -> icalendar::Calendar {
    let event = icalendar::Event::new()
        .uid(id)
        .summary(name)
        .starts(start)
        .ends(end)
        .description(description)
        .done();

    let calendar = icalendar::Calendar::new()
        .timezone("UTC")
        .push(event)
        .done();

    calendar
}
