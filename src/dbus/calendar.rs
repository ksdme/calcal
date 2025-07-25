#[zbus::proxy(
    default_service = "org.gnome.evolution.dataserver.Calendar8",
    default_path = "/org/gnome/evolution/dataserver/CalendarFactory",
    interface = "org.gnome.evolution.dataserver.CalendarFactory"
)]
pub trait CalendarFactory {
    // This call returns the object path and the bus name to query a specific
    // calendar using its UID. Ideally, the return type should be os, but it is
    // reported as ss during introspection.
    async fn open_calendar(&self, uid: &str) -> zbus::Result<(String, String)>;
}
