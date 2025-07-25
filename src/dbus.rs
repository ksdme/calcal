use std::collections::HashMap;
use zbus::zvariant;

#[zbus::proxy(
    default_service = "org.gnome.evolution.dataserver.Sources5",
    default_path = "/org/gnome/evolution/dataserver/SourceManager",
    interface = "org.freedesktop.DBus.ObjectManager"
)]
pub trait SourcesObjectManager {
    // Returns the source objects that are created and managed at runtime.
    // Uses the standard org.freedesktop.DBus.ObjectManager.
    async fn get_managed_objects(
        &self,
    ) -> zbus::Result<
        HashMap<zvariant::OwnedObjectPath, HashMap<String, HashMap<String, zvariant::OwnedValue>>>,
    >;
}
