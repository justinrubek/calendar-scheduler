use axum::{
    routing::{get, post},
    Router,
};
use caldav_utils::{
    availability::get_availability,
    caldav::client::{DavClient, DavCredentials},
};
use chrono::TimeZone;
use clap::Parser;
use commands::ServerCommands;
use reqwest::Client;
use rrule::{RRuleSet, Tz};
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

    let caldav_state = CaldavAvailability::new(
        "meeting_availability".to_string(),
        "meeting_booked".to_string(),
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
                    for event in events {
                        tracing::info!("event: {:?}", event);
                    }
                }
            }
        }
    }

    // caldav_experiment().await?;
    // scheduler_api(caldav_state).await?;
    // availability_experiment(caldav_state).await?;

    Ok(())
}

#[allow(dead_code)]
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
async fn caldav_experiment() -> Result<(), Box<dyn std::error::Error>> {
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
        // .find(|c| c.display_name == "testing")
        .expect("no availability calendar found");
    info!(
        "found availability calendar: {}",
        availability_calendar.display_name
    );

    let start = chrono::Utc::now();
    let end = start + chrono::Duration::days(2);
    info!("getting events from {} to {}", start, end);

    let events = availability_calendar
        .get_events(&client, start, end)
        .await?;
    info!("found {} events", events.len());

    for (ei, event) in events.iter().enumerate() {
        let comps = &event.ical.components;
        info!("event {ei}: {:?}", comps);
        for (ci, comp) in comps.iter().enumerate() {
            let ev1 = comp.as_event().unwrap();
            info!("comp {}: {:?}", ci, ev1);
        }
        /*
        let ev2 = comps.get(1).unwrap();
        info!("event: {:?}", ev1);
        info!("event: {:?}", ev2);

        info!("event: {}", ev1.get_start());
        */
    }

    // let rrule: RRuleSet = "DTSTART:20230105T130000\nDTEND:20230105T160000\nRRULE:FREQ=WEEKLY".parse().unwrap();
    let rrule: RRuleSet = "DTSTART:20230106T133500Z\nRRULE:FREQ=DAILY;COUNT=3"
        .parse()
        .unwrap();
    // let tz_start = Tz::UTC.with_ymd_and_hms(2023, 1, 5, 13, 0, 0).unwrap();
    let tz_start = Tz::UTC.from_utc_datetime(&start.naive_utc());
    let tz_end = Tz::UTC.from_utc_datetime(&end.naive_utc());
    let (detected_events, _) = rrule.after(tz_start).before(tz_end).all(100);

    info!("found {} detected events", detected_events.len());
    info!("detected events: {:?}", detected_events);

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
