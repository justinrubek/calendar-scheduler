use axum::{
    routing::{get, post},
    Router,
};
use caldav_utils::{
    availability::get_availability,
    caldav::client::{DavClient, DavCredentials},
};
use clap::Parser;
use commands::ServerCommands;
use reqwest::Client;
use scheduling_api::{get_calendars, get_now, request_availability, state::CaldavAvailability};
use std::net::SocketAddr;
use tracing::info;

mod commands;
use crate::commands::{CalendarCommands, Commands};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // read configuration
    let username = std::env::var("CALDAV_USERNAME").expect("CALDAV_USERNAME not set");
    let password = std::env::var("CALDAV_PASSWORD").expect("CALDAV_PASSWORD not set");
    let credentials = DavCredentials::new(username.to_string(), password.to_string());

    let url = std::env::var("CALDAV_URL").expect("CALDAV_URL not set");

    let dav_client = DavClient::new(url.to_string(), credentials);

    let availability_calendar =
        std::env::var("AVAILABLE_CALENDAR").expect("AVAILABLE_CALENDAR not set");
    let booked_calendar = std::env::var("BOOKED_CALENDAR").expect("BOOKED_CALENDAR not set");

    let caldav_state = CaldavAvailability::new(
        availability_calendar.to_string(),
        booked_calendar.to_string(),
        dav_client,
    );

    // process commands
    let args = commands::Args::parse();
    match args.command {
        Commands::Server(server) => {
            let cmd = server.command;
            match cmd {
                ServerCommands::Start => scheduler_api(caldav_state).await?,
            }
        }
        Commands::Calendar(calendar) => {
            let cmd = calendar.command;
            match cmd {
                CalendarCommands::Create(create) => {
                    let client = Client::new();
                    let mut principal = caldav_state.davclient().get_principal(&client).await?;
                    let calendar = principal
                        .create_calendar_mkcol(&client, &create.name)
                        .await?;
                    println!("Created calendar: {}", calendar.path);
                }
                CalendarCommands::List => {
                    let client = Client::new();
                    let mut principal = caldav_state.davclient().get_principal(&client).await?;
                    let calendars = principal.get_calendars(&client).await?;
                    for calendar in calendars {
                        println!("calendar {} at {}", calendar.display_name, calendar.path);
                    }
                }
                CalendarCommands::ListEvents(list) => {
                    let client = Client::new();
                    let mut principal = caldav_state.davclient().get_principal(&client).await?;
                    let calendar = principal.get_calendar(&client, &list.name).await?;
                    let events = calendar.get_events(&client, list.start, list.end).await?;
                    tracing::info!("Found {} events", events.len());
                    for event in events {
                        tracing::info!("event: {:?}", event);
                    }
                }
            }
        }
    }

    Ok(())
}

async fn scheduler_api(caldav_state: CaldavAvailability) -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("PORT")
        .ok()
        .and_then(|it| it.parse().ok())
        .unwrap_or(8000);

    let app = Router::new()
        .route("/now", get(get_now))
        // POST since JS doesn't support body in GET
        .route("/availability", post(request_availability))
        .with_state(caldav_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Listening on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

#[allow(dead_code)]
async fn availability_experiment(
    caldav_state: CaldavAvailability,
) -> Result<(), Box<dyn std::error::Error>> {
    let start = chrono::Utc::now();
    let end = start + chrono::Duration::days(7);
    info!("getting availability from {} to {}", start, end);

    let client = reqwest::Client::new();
    let (availability_calendar, booked_calendar) = get_calendars(&client, caldav_state).await?;

    let granularity = chrono::Duration::minutes(30);

    let availability = get_availability(
        &client,
        &availability_calendar,
        &booked_calendar,
        start,
        end,
        granularity,
    )
    .await?;

    // count the number of available slots (true values in availability.matrix vec)
    // info!("found {} availability slots", availability.matrix.len());
    let slots = availability.matrix.iter().filter(|&&x| x).count();
    info!("found {} availability slots", slots);

    Ok(())
}
