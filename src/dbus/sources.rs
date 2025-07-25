use gio::glib;
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

#[derive(Debug)]
pub struct Source {
    pub object_path: String,

    pub uid: String,
    pub display_name: Option<String>,

    pub has_calendar: bool,
}

impl<'a> SourcesObjectManagerProxy<'a> {
    // Returns a list of all the EDS sources from the dbus.
    pub async fn list_sources(&self) -> zbus::Result<Vec<Source>> {
        let mut sources: Vec<Source> = vec![];

        let objects = self.get_managed_objects().await?;
        for (object_path, object_value) in objects.iter() {
            // We only care about the Source interface, the other ones allow you to change the source.
            if let Some(source_value) = object_value.get("org.gnome.evolution.dataserver.Source") {
                let Some(uid) = source_value
                    .get("UID")
                    .and_then(|uid| uid.downcast_ref::<String>().ok())
                else {
                    continue;
                };

                let data = source_value
                    .get("Data")
                    .and_then(|data| data.downcast_ref::<&str>().ok())
                    .and_then(|data| {
                        let key_file = glib::KeyFile::new();
                        match key_file.load_from_data(data, glib::KeyFileFlags::NONE) {
                            Ok(()) => Some(key_file),
                            _ => None,
                        }
                    });

                sources.push(Source {
                    object_path: object_path.to_string(),

                    uid: uid,
                    display_name: data
                        .as_ref()
                        .and_then(|value| value.string("Data Source", "DisplayName").ok())
                        .and_then(|name| Some(name.to_string())),

                    has_calendar: data
                        .as_ref()
                        .and_then(|value| Some(value.has_group("Calendar")))
                        .unwrap_or_default(),
                });
            }
        }

        Ok(sources)
    }
}
