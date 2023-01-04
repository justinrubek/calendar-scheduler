use reqwest::Client;
use tracing::info;

use caldav_utils::client::{DavClient, DavCredentials};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let username = std::env::var("CALDAV_USERNAME").expect("CALDAV_USERNAME not set");
    let password = std::env::var("CALDAV_PASSWORD").expect("CALDAV_PASSWORD not set");
    let credentials = DavCredentials::new(username.to_string(), password.to_string());

    let url = std::env::var("CALDAV_URL").expect("CALDAV_URL not set");

    let dav_client = DavClient::new(url.to_string(), credentials);
    let client = Client::new();
    let mut principal = dav_client.get_principal(&client).await?;

    let calendars = principal.get_calendars(&client).await?;

    let availability_calendar = calendars
        .iter()
        .find(|c| c.display_name == "meeting_availability")
        .expect("no availability calendar found");
    info!(
        "found availability calendar: {}",
        availability_calendar.display_name
    );

    let start = chrono::Utc::now();
    let end = start + chrono::Duration::days(7);

    let events = availability_calendar
        .get_events(&client, start, end)
        .await?;
    info!("found {} events", events.len());
    info!("first event: {:?}", events[0]);

    Ok(())
}
