use reqwest::Client;
use cli::caldav::{
    client::{DavClient, DavCredentials},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let username = std::env::var("CALDAV_USERNAME").expect("CALDAV_USERNAME not set");
    let password = std::env::var("CALDAV_PASSWORD").expect("CALDAV_PASSWORD not set");
    let credentials = DavCredentials::new(username.to_string(), password.to_string());

    let url = std::env::var("CALDAV_URL").expect("CALDAV_URL not set");

    let dav_client = DavClient::new(url.to_string(), credentials);
    let client = Client::new();
    let mut principal = dav_client.get_principal(&client).await?;

    let calendars = principal.get_calendars(&client).await?;
    dbg!(calendars);

    Ok(())
}
