use crate::format;

#[derive(Debug)]
pub struct Event {
    pub ical: icalendar::Calendar,
}

#[derive(Debug)]
pub enum Property {
    DateTime(chrono::DateTime<chrono::Utc>),
    String(String),
}

impl Event {
    pub fn new(ical: icalendar::Calendar) -> Event {
        Event { ical }
    }

    pub fn add_property(&mut self, key: &str, property: Property) {
        let property = match property {
            Property::DateTime(dt) => {
                let dt_str = format!("{}", dt.format(format::DATETIME));
                icalendar::Property::new(key, &dt_str)
            }
            Property::String(s) => icalendar::Property::new(key, &s),
        };

        self.ical.append_property(property);
    }
}

