use std::collections::HashMap;
use zbus::zvariant;

#[zbus::proxy(
    default_service = "org.gnome.evolution.dataserver.Sources5",
    default_path = "/org/gnome/evolution/dataserver/SourceManager",
    interface = "org.freedesktop.DBus.ObjectManager"
)]
pub trait Sources {
    // Returns the source objects that are created and managed at runtime.
    // Uses the standard org.freedesktop.DBus.ObjectManager.
    async fn get_managed_objects(
        &self,
    ) -> zbus::Result<
        HashMap<zvariant::OwnedObjectPath, HashMap<String, HashMap<String, zvariant::OwnedValue>>>,
    >;
}

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

#[zbus::proxy(
    default_service = "org.gnome.evolution.dataserver.Calendar8",
    interface = "org.gnome.evolution.dataserver.Calendar"
)]
pub trait Calendar {
    // This call returns ics_objects based on a query string. The object path
    // to query should be based on the calendar.
    async fn get_object_list(&self, q: &str) -> zbus::Result<Vec<String>>;
}
