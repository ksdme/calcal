use anyhow::Context;

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

impl<'a> CalendarProxy<'a> {
    // Returns a list of events between a starts and end date time in the local timezone.
    pub async fn list_events(&self, starts: jiff::Zoned, ends: jiff::Zoned) -> anyhow::Result<()> {
        let q = format!(
            "
            (occur-in-time-range?
                (make-time \"{}\")
                (make-time \"{}\"))
            ",
            starts.strftime("%Y%m%dT%H%M%S"),
            ends.strftime("%Y%m%dT%H%M%S")
        );

        let events = self
            .get_object_list(&q)
            .await
            .context("Could not query events")?;

        Ok(())
    }

    // Returns a list of events that are scheduled for today.
    pub async fn list_today_events(&self) -> anyhow::Result<()> {
        let now = jiff::Zoned::now();

        let starts = now
            .start_of_day()
            .context("Could not determine start of today")?;

        let ends = now
            .tomorrow()
            .context("Could not determine date of tomorrow")?
            .start_of_day()
            .context("Could not determine start of tomorrow")?;

        self.list_events(starts, ends).await
    }
}
