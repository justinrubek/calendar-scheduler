#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand, Debug)]
pub(crate) enum Commands {
    Server(Server),
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
    Create(CreateCalendarCommand),
    List,
    ListEvents(ListEventsCommand),
}

#[derive(clap::Args, Debug)]
pub(crate) struct CreateCalendarCommand {
    pub name: String,
}

#[derive(clap::Args, Debug)]
pub(crate) struct ListEventsCommand {
    pub name: String,
    pub start: chrono::DateTime<chrono::Utc>,
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
    Start,
}
