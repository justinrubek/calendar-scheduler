use minidom::Element;
use reqwest::Method;
use tracing::info;
use url::Url;

use super::format;
use super::client::DavClient;
use super::util::find_elements;

#[derive(Clone, Debug)]
pub struct Calendar {
    client: DavClient,
    url: Url,
    pub path: String,

    pub display_name: String,
}

impl Calendar {
    pub fn new(
        client: DavClient,
        url: Url,
        path: String,
        display_name: String,
    ) -> Calendar {
        Calendar {
            client,
            url,
            path,
            display_name,
        }
    }

    pub async fn get_events(
        &self,
        client: &reqwest::Client,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Format timestaps for caldav e.g. "20230108T000000Z";
        let start_str = start.format(format::DATETIME);
        let end_str = end.format(format::DATETIME);

        let body = format!(r#"
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
        "#);

        let mut url = self.url.clone();
        url.set_path(&self.path);
        let method = Method::from_bytes(b"REPORT")?;

        info!("fetching events from {}", url);

        let req = client
            .request(method, url.as_str())
            .header("Depth", 1)
            .header("Content-Type", "application/xml")
            .basic_auth(self.client.credentials.username.clone(), Some(self.client.credentials.password.clone()))
            .body(body);

        tracing::debug!("request: {:?}", req);

        let res = req.send().await?;

        tracing::debug!("response: {:?}", res);

        let text = res.text().await?;

        let root: Element = text.parse()?;
        let data = find_elements(&root, "calendar-data".to_string());
        let events = data.iter().map(|d| d.text()).collect();

        Ok(events)
    }
}
