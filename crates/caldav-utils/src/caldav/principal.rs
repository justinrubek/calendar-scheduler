use minidom::Element;
use reqwest::{
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    Client, Method, Result,
};
use url::Url;

use crate::error::{CaldavError, CaldavResult};
use crate::util::{find_element, find_elements};

use super::calendar::Calendar;
use super::client::DavClient;

static HOMESET_BODY: &str = r#"
    <d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav" >
      <d:self/>
      <d:prop>
        <c:calendar-home-set />
      </d:prop>
    </d:propfind>
"#;

static CALENDAR_BODY: &str = r#"
    <d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav" xmlns:ical="http://apple.com/ns/ical/" >
       <d:prop>
         <d:displayname />
         <ical:calendar-timezone />
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

        tracing::debug!("principal response: {}", text);

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
        // short-circuit if we already have the calendars
        if !self.calendars.is_empty() {
            return Ok(self.calendars.clone());
        }

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

        tracing::debug!("calendar response: {}", text);

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

                let timezone = find_element(response, "calendar-timezone".to_string())
                    .expect("failed to find calendar-timezone")
                    .text();

                let href = find_element(response, "href".to_string())
                    .expect("failed to find href")
                    .text();

                Some(Calendar::new(
                    self.client.clone(),
                    self.url.clone(),
                    href,
                    displayname,
                    Some(timezone),
                ))
            })
            .collect();

        self.calendars = calendars.clone();
        Ok(calendars)
    }

    pub async fn get_calendar(
        &mut self,
        client: &reqwest::Client,
        calendar_name: &str,
    ) -> CaldavResult<Calendar> {
        let calendars = self.get_calendars(client).await?;
        let calendar = calendars
            .iter()
            .find(|c| c.display_name == calendar_name)
            .ok_or_else(|| CaldavError::CalendarNotFound {
                calendar_name: calendar_name.to_string(),
            })?;
        Ok(calendar.clone())
    }

    pub async fn create_calendar_mkcol(
        &mut self,
        client: &reqwest::Client,
        calendar_name: &str,
    ) -> CaldavResult<Calendar> {
        let mut url = match &self.homeset_url {
            Some(url) => url.clone(),
            None => self.get_home_set(client).await?,
        };
        // generate a unique id for the calendar
        let id = ksuid::Ksuid::generate().to_base62();
        url.set_path(&format!("{}{}/", url.path(), id));
        let url = format!("{url}");
        tracing::info!("url: {}", url);

        let method = Method::from_bytes(b"MKCOL").expect("failed to create MKCOL method");

        let body = format!(
            r#"<?xml version="1.0"?>
            <mkcol xmlns="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav" xmlns:ICAL="http://apple.com/ns/ical/">
              <set>
                <prop>
                  <resourcetype>
                    <collection />
                    <C:calendar />
                  </resourcetype>
                  <C:supported-calendar-component-set>
                    <C:comp name="VEVENT" />
                    <C:comp name="VJOURNAL" />
                    <C:comp name="VTODO" />
                  </C:supported-calendar-component-set>
                  <displayname>{calendar_name}</displayname>
                  <ICAL:calendar-timezone>UTC</ICAL:calendar-timezone>
                  <ICAL:calendar-color>#ff0000</ICAL:calendar-color>
                </prop>
              </set>
            </mkcol>
            "#
        );

        tracing::debug!("calendar: {}", body);

        // Make sure to set the calendar timezone to UTC
        let res = client
            .request(method, url.clone())
            .header(CONTENT_TYPE, "application/xml")
            .header(CONTENT_LENGTH, body.len())
            .basic_auth(
                &self.client.credentials.username,
                Some(&self.client.credentials.password),
            )
            .body(body)
            .send()
            .await?;

        tracing::debug!("response: {:?}", res);
        let res = match res.status() {
            reqwest::StatusCode::CREATED => res,
            _ => {
                let error = res.text().await?;
                return Err(CaldavError::ServerResponse(error));
            }
        };
        let text = res.text().await?;

        tracing::debug!("calendar response: {}", text);

        Ok(Calendar::new(
            self.client.clone(),
            self.url.clone(),
            // TODO: this is wrong, need to get the href from the response
            url,
            calendar_name.to_string(),
            Some("UTC".to_string()),
        ))
    }
}
