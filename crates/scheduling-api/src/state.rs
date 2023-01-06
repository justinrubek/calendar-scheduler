use caldav_utils::caldav::client::DavClient;

/// The details needed to connect to a caldav server
/// and find relevant calendars to determine availability.
#[derive(Clone, Debug)]
pub struct CaldavAvailability {
    pub(crate) availability_calendar: String,
    pub(crate) booked_calendar: String,
    pub(crate) davclient: DavClient,
}

impl CaldavAvailability {
    pub fn new(
        availability_calendar: String,
        booked_calendar: String,
        davclient: DavClient,
    ) -> Self {
        Self {
            availability_calendar,
            booked_calendar,
            davclient,
        }
    }
}
