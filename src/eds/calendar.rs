use crate::eds::{event::Event, ipc};
use anyhow::Context;
use calcard::icalendar;
use chrono::{Days, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use gio::glib;

#[derive(Debug)]
pub struct Calendar<'a> {
    conn: &'a zbus::Connection,

    pub uid: String,
    pub display_name: Option<String>,
}

impl<'a> Calendar<'a> {
    // Returns a list of all the calendars that were found on the Evolution Data Server.
    pub async fn fetch_all(conn: &'a zbus::Connection) -> anyhow::Result<Vec<Self>> {
        let sources_proxy = ipc::SourcesProxy::new(conn)
            .await
            .context("Could not build sources proxy")?;

        let sources = sources_proxy
            .get_managed_objects()
            .await
            .context("Could not fetch the list of sources")?;

        let mut calendars: Vec<Self> = vec![];
        for (_, object_value) in sources.iter() {
            // The other interfaces are meant for mutation and stuff.
            let Some(source_value) = object_value.get("org.gnome.evolution.dataserver.Source")
            else {
                continue;
            };

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

            // Filter for sources that have a calendars attached to them.
            if !data
                .as_ref()
                .and_then(|value| Some(value.has_group("Calendar")))
                .unwrap_or_default()
            {
                continue;
            }

            calendars.push(Self {
                conn: conn,
                uid: uid,
                display_name: data
                    .as_ref()
                    .and_then(|value| value.string("Data Source", "DisplayName").ok())
                    .and_then(|name| Some(name.to_string())),
            });
        }

        Ok(calendars)
    }

    // Returns a list of all the events found on this calendar on the EDS.
    async fn fetch_events(
        &self,
        starts: chrono::DateTime<chrono::Local>,
        ends: chrono::DateTime<chrono::Local>,
    ) -> anyhow::Result<Vec<super::event::Event>> {
        let calendar_factory_proxy = ipc::CalendarFactoryProxy::new(self.conn)
            .await
            .context("Could not build calendar factory proxy")?;

        let (calendar_path, _) = calendar_factory_proxy
            .open_calendar(&self.uid)
            .await
            .context("Could not query calendar")?;

        let calendar_proxy = ipc::CalendarProxy::builder(self.conn)
            .path(calendar_path)
            .context("Could not set path on calendar proxy")?
            .build()
            .await
            .context("Could not build calendar proxy")?;

        let q = format!(
            "
            (occur-in-time-range?
                (make-time \"{}\")
                (make-time \"{}\"))
            ",
            starts.format("%Y%m%dT%H%M%S"),
            ends.format("%Y%m%dT%H%M%S")
        );

        let vevent_result = calendar_proxy
            .get_object_list(&q)
            .await
            .context("Could not query events")?;

        let vevents: Vec<icalendar::ICalendarComponent> = vevent_result
            .iter()
            .filter_map(|item| icalendar::ICalendar::parse(item).ok())
            .flat_map(|cal| -> Vec<icalendar::ICalendarComponent> {
                cal.components
                    .into_iter()
                    .filter(|item| item.component_type == icalendar::ICalendarComponentType::VEvent)
                    .collect()
            })
            .collect();

        Ok(vevents
            .iter()
            .flat_map(|vevent| {
                // TODO: Is this enough to know if we should be expanding?
                if vevent
                    .property(&icalendar::ICalendarProperty::Rrule)
                    .is_some()
                {
                    expand_events(vevent, starts, ends).unwrap_or_default()
                } else {
                    vec![Event::from(vevent)]
                }
            })
            .collect())
    }

    // Returns a list of events that were scheduled from start of yesterday to end of tomorrow.
    pub async fn fetch_near_events(&self) -> anyhow::Result<Vec<super::event::Event>> {
        let now = chrono::Local::now();

        let starts = now
            .with_time(chrono::NaiveTime::default())
            .earliest()
            .and_then(|dt| dt.checked_sub_days(Days::new(1)))
            .context("Could not determine start of yesterday")?;

        let ends = now
            .with_time(chrono::NaiveTime::default())
            .earliest()
            .and_then(|dt| dt.checked_add_days(chrono::Days::new(2)))
            .context("Could not determine the start of day after tomorrow")?;

        self.fetch_events(starts, ends).await
    }
}

// Returns a recurrence expanded list of events.
fn expand_events(
    root: &icalendar::ICalendarComponent,
    starts: chrono::DateTime<chrono::Local>,
    ends: chrono::DateTime<chrono::Local>,
) -> Option<Vec<Event>> {
    let mut rule = String::new();

    root.property(&calcard::icalendar::ICalendarProperty::Dtstart)
        .and_then(|it| it.write_to(&mut rule).ok());

    root.property(&calcard::icalendar::ICalendarProperty::Rrule)
        .and_then(|it| it.write_to(&mut rule).ok());

    root.property(&calcard::icalendar::ICalendarProperty::Rdate)
        .and_then(|it| it.write_to(&mut rule).ok());

    root.property(&calcard::icalendar::ICalendarProperty::Exdate)
        .and_then(|it| it.write_to(&mut rule).ok());

    root.property(&calcard::icalendar::ICalendarProperty::Exrule)
        .and_then(|it| it.write_to(&mut rule).ok());

    if let Ok(rrule) = rule.as_str().parse::<rrule::RRuleSet>() {
        let starts = starts
            .naive_local()
            .and_local_timezone(rrule::Tz::Local(chrono::Local))
            .earliest()?;

        let ends = ends
            .naive_local()
            .and_local_timezone(rrule::Tz::Local(chrono::Local))
            .earliest()?;

        Some(Event::from_recurrences(
            &root,
            rrule.after(starts).before(ends).all(32),
        ))
    } else {
        None
    }
}
