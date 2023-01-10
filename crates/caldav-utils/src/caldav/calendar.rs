use icalendar::{Component, EventLike};
use minidom::Element;
use reqwest::Method;
use url::Url;

use crate::error::{CaldavError, CaldavResult};
use crate::format;
use crate::util::find_elements;

use super::client::DavClient;
use super::event::Event;

#[derive(Clone, Debug)]
pub struct Calendar {
    client: DavClient,
    url: Url,
    pub path: String,

    pub display_name: String,
    pub timezone: Option<String>,
}

impl Calendar {
    pub fn new(
        client: DavClient,
        url: Url,
        path: String,
        display_name: String,
        timezone: Option<String>,
    ) -> Calendar {
        Calendar {
            client,
            url,
            path,
            display_name,
            timezone,
        }
    }

    pub async fn get_events(
        &self,
        client: &reqwest::Client,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> CaldavResult<Vec<Event>> {
        // Format timestaps for caldav e.g. "20230108T000000Z";
        let start_str = start.format(format::DATETIME);
        let end_str = end.format(format::DATETIME);

        let body = format!(
            r#"
            <c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
              <d:prop>
                <d:getetag />
                <c:calendar-data />
              </d:prop>
              <c:filter>
                <c:comp-filter name="VCALENDAR">
                  <c:comp-filter name="VEVENT" >
                    <c:time-range start="{start_str}" end="{end_str}" />
                  </c:comp-filter>
                </c:comp-filter>
              </c:filter>
            </c:calendar-query>
        "#
        );

        let mut url = self.url.clone();
        url.set_path(&self.path);
        let method = Method::from_bytes(b"REPORT")?;

        tracing::debug!("fetching events from {}", url);

        let req = client
            .request(method, url.as_str())
            .header("Depth", 1)
            .header("Content-Type", "application/xml")
            .basic_auth(
                self.client.credentials.username.clone(),
                Some(self.client.credentials.password.clone()),
            )
            .body(body);

        tracing::debug!("request: {:?}", req);

        let res = req.send().await?;

        tracing::debug!("response: {:?}", res);

        let text = res.text().await?;

        let root: Element = text.parse()?;
        let data = find_elements(&root, "calendar-data".to_string());
        let events: Vec<_> = data
            .iter()
            .map(|d| d.text())
            .map(|t| {
                let ical = icalendar::parser::read_calendar(&t).expect("failed to parse ical");
                Event { ical: ical.into() }
            })
            .collect();

        Ok(events)
    }

    pub async fn create_event(
        &self,
        client: &reqwest::Client,
        name: &str,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> CaldavResult<Event> {
        let id = ksuid::Ksuid::generate().to_base62();

        let event = icalendar::Event::new()
            .uid(&id)
            .summary(name)
            .starts(start)
            .ends(end)
            .done();

        let calendar = icalendar::Calendar::new()
            .timezone("UTC")
            .push(event)
            .done();

        tracing::debug!("creating event: {:?}", calendar.to_string());

        // Perform an HTTP PUT request to create a new event
        let mut url = self.url.clone();
        url.set_path(&format!("{}/{}.ics", self.path, id));

        let method = Method::PUT;

        let res = client
            .request(method, url.as_str())
            .header("Content-Type", "text/calendar")
            .basic_auth(
                self.client.credentials.username.clone(),
                Some(self.client.credentials.password.clone()),
            )
            .body(calendar.to_string())
            // .body(fake_event)
            .send()
            .await?;

        let res = match res.status() {
            reqwest::StatusCode::CREATED => res,
            _ => {
                let error = res.text().await?;
                return Err(CaldavError::ServerResponse(error));
            }
        };

        tracing::debug!("response: {:?}", res);

        Ok(Event { ical: calendar })
    }
}
