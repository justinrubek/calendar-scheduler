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
