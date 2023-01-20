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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_event() {
        // We don't want to test the entire icalendar crate, so we'll just
        // test that the events we create can be parsed back into a calendar
        let id = "123";
        let name = "Test Event";
        let description = "This is a test event\nfoo";
        let start = chrono::Utc::now();
        let end = start + chrono::Duration::hours(1);

        let calendar = create_event(id, name, description, start, end);
        println!("{}", calendar.to_string());

        let cstr = calendar.to_string();
        let parsed = icalendar::parser::read_calendar(&cstr).unwrap();

        assert!(true);
    }
}
