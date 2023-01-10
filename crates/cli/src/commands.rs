#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand, Debug)]
pub(crate) enum Commands {
    /// commands for running the scheduling server
    Server(Server),
    /// commands for interacting with a calendar on a caldav server
    Calendar(Calendar),
}

#[derive(clap::Args, Debug)]
#[command(args_conflicts_with_subcommands = true)]
pub(crate) struct Calendar {
    #[clap(subcommand)]
    pub command: CalendarCommands,
}

#[derive(clap::Subcommand, Debug)]
pub(crate) enum CalendarCommands {
    /// create a new calendar
    Create(CreateCalendarCommand),
    /// list all calendars
    List,
    /// list events in a calendar between two datetimes
    ListEvents(ListEventsCommand),
    /// get the availability of a calendar between two datetimes
    Availability(AvailabilityCommand),
}

#[derive(clap::Args, Debug)]
pub(crate) struct CreateCalendarCommand {
    pub name: String,
}

#[derive(clap::Args, Debug)]
pub(crate) struct ListEventsCommand {
    /// the name of the calendar
    pub name: String,
    /// the start of the time range
    pub start: chrono::DateTime<chrono::Utc>,
    /// the end of the time range
    pub end: chrono::DateTime<chrono::Utc>,
}

#[derive(clap::Args, Debug)]
#[command(args_conflicts_with_subcommands = true)]
pub(crate) struct Server {
    #[clap(subcommand)]
    pub command: ServerCommands,
}

#[derive(clap::Subcommand, Debug)]
pub(crate) enum ServerCommands {
    /// start the scheduling http server
    Start,
}

#[derive(clap::Args, Debug)]
pub(crate) struct AvailabilityCommand {
    /// the name of the calendar
    pub name: String,
    /// the start of the time range
    pub start: chrono::DateTime<chrono::Utc>,
    /// the end of the time range
    pub end: chrono::DateTime<chrono::Utc>,
    /// the amount of time to subdivide the availability into.
    /// e.g. if this is 30 minutes, then the availability matrix will contain
    /// a slot for every 30 minutes between start and end.
    #[clap(long, default_value = "30")]
    pub granularity: i64,
}
