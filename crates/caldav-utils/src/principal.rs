use minidom::Element;
use reqwest::{header::CONTENT_TYPE, Client, Method, Result};
use url::Url;

use super::calendar::Calendar;
use super::client::DavClient;
use super::util::{find_element, find_elements};

static HOMESET_BODY: &str = r#"
    <d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav" >
      <d:self/>
      <d:prop>
        <c:calendar-home-set />
      </d:prop>
    </d:propfind>
"#;

static CALENDAR_BODY: &str = r#"
    <d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav" >
       <d:prop>
         <d:displayname />
         <d:resourcetype />
         <c:supported-calendar-component-set />
       </d:prop>
    </d:propfind>
"#;

/// A CalDav Principal
/// This represents a distinct human actor that can be authenticated
/// and access resources on a CalDav server.
#[derive(Debug)]
pub struct Principal {
    url: Url,
    homeset_url: Option<Url>,
    calendars: Vec<Calendar>,
    client: DavClient,
}

impl Principal {
    pub fn new(client: DavClient, url: Url) -> Principal {
        Principal {
            client,
            url,
            homeset_url: None,
            calendars: Vec::new(),
        }
    }

    pub async fn get_home_set(&mut self, client: &Client) -> Result<Url> {
        let method = Method::from_bytes(b"PROPFIND").expect("failed to create PROPFIND method");

        let res = client
            .request(method, self.url.as_str())
            .header("Depth", "0")
            .header(CONTENT_TYPE, "application/xml")
            .basic_auth(
                &self.client.credentials.username,
                Some(&self.client.credentials.password),
            )
            .body(HOMESET_BODY)
            .send()
            .await?;

        let text = res.text().await?;

        let root: Element = text.parse().expect("failed to parse xml");
        let homeset =
            find_element(&root, "response".to_string()).expect("failed to find calendar-home-set");
        let homeset_href =
            find_element(homeset, "href".to_string()).expect("failed to find homeset's href");
        let href = homeset_href.text();

        let mut url = self.url.clone();
        url.set_path(&href);

        self.homeset_url = Some(url.clone());
        Ok(url)
    }

    pub async fn get_calendars(&mut self, client: &Client) -> Result<Vec<Calendar>> {
        let homeset_url = match &self.homeset_url {
            Some(url) => url.clone(),
            None => self.get_home_set(client).await?,
        };
        tracing::debug!("getting calendars from {}", homeset_url);

        let method = Method::from_bytes(b"PROPFIND").expect("failed to create PROPFIND method");

        let res = client
            .request(method, homeset_url.clone())
            .header("Depth", "1")
            .header(CONTENT_TYPE, "application/xml")
            .basic_auth(
                &self.client.credentials.username,
                Some(&self.client.credentials.password),
            )
            .body(CALENDAR_BODY)
            .send()
            .await?;

        let text = res.text().await?;

        tracing::debug!("response: {}", text);

        let root: Element = text.parse().expect("failed to parse xml");
        let responses = find_elements(&root, "response".to_string());
        let calendars: Vec<_> = responses
            .iter()
            .filter_map(|response| {
                let displayname = find_element(response, "displayname".to_string())
                    .expect("failed to find displayname")
                    .text();
                if displayname.is_empty() {
                    return None;
                }

                let href = find_element(response, "href".to_string())
                    .expect("failed to find href")
                    .text();

                Some(Calendar::new(
                    self.client.clone(),
                    self.url.clone(),
                    href,
                    displayname,
                ))
            })
            .collect();

        self.calendars = calendars.clone();
        Ok(calendars)
    }
}
