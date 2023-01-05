use caldav_utils::client::DavClient;

#[derive(Clone, Debug)]
pub struct CaldavAvailability {
    availability_calendar: String,
    booked_calendar: String,
    davclient: DavClient,
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
