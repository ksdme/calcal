use anyhow::Context;
use calcard::icalendar;
use chrono::{Datelike, TimeZone};

#[derive(Debug)]
pub struct Event {
    pub uid: Option<String>,
    pub status: Option<icalendar::ICalendarStatus>,

    pub title: Option<String>,
    pub description: Option<String>,

    pub starts: Option<chrono::DateTime<rrule::Tz>>,
    pub ends: Option<chrono::DateTime<rrule::Tz>>,
}

impl From<&icalendar::ICalendarComponent> for Event {
    fn from(component: &icalendar::ICalendarComponent) -> Self {
        Self {
            uid: component.uid().map(|uid| uid.to_owned()),
            status: component.status().map(|status| status.clone()),

            title: str_property(&component, &icalendar::ICalendarProperty::Summary),
            description: str_property(&component, &icalendar::ICalendarProperty::Description),

            starts: match dt_property(&component, &icalendar::ICalendarProperty::Dtstart) {
                Some(Ok(dtstarts)) => Some(dtstarts),
                _ => None,
            },
            ends: match dt_property(&component, &icalendar::ICalendarProperty::Dtend) {
                Some(Ok(dtends)) => Some(dtends),
                _ => None,
            },
        }
    }
}

impl Event {
    pub fn from_recurrences(
        component: &icalendar::ICalendarComponent,
        recurrences: rrule::RRuleResult,
    ) -> Vec<Self> {
        let root = Event::from(component);

        let duration = match (root.starts, root.ends) {
            (Some(dtstarts), Some(dtends)) => Some(dtends.to_utc() - dtstarts.to_utc()),
            _ => None,
        };

        recurrences
            .dates
            .into_iter()
            .map(|starts| Self {
                uid: root.uid.clone(),
                status: root.status.clone(),

                title: root.title.clone(),
                description: root.description.clone(),

                starts: Some(starts),
                ends: duration.map(|duration| starts + duration),
            })
            .collect()
    }
}

// Load up a property from the calendar component as a string value.
fn str_property(
    component: &icalendar::ICalendarComponent,
    property: &icalendar::ICalendarProperty,
) -> Option<String> {
    Some(
        component
            .property(property)?
            .values
            .first()?
            .as_text()?
            .to_owned(),
    )
}

// Transform the date time value from the calendar component while taking the
// timezone into account.
fn dt_property(
    component: &icalendar::ICalendarComponent,
    property: &icalendar::ICalendarProperty,
) -> Option<anyhow::Result<chrono::DateTime<rrule::Tz>>> {
    let property = component.property(property)?;

    // NaiveDateTime
    let now = chrono::Local::now();
    let dt = property.values.first()?.as_partial_date_time()?;
    let dt = chrono::NaiveDate::from_ymd_opt(
        dt.year.map(|y| y as i32).unwrap_or(now.year()),
        dt.month.map(|m| m as u32).unwrap_or(now.month()),
        dt.day.map(|d| d as u32).unwrap_or(now.day()),
    )?
    .and_time(chrono::NaiveTime::from_hms_opt(
        dt.hour.map(|h| h as u32).unwrap_or_default(),
        dt.minute.map(|m| m as u32).unwrap_or_default(),
        dt.second.map(|s| s as u32).unwrap_or_default(),
    )?);

    // Timezone
    let tz = property.params.first().and_then(|value| match value {
        icalendar::ICalendarParameter::Tzid(tz) => Some(tz),
        _ => None,
    });

    // DateTime
    if let Some(tz) = tz {
        match tz
            .as_str()
            .parse::<chrono_tz::Tz>()
            .context("Could not parse timezone")
        {
            Ok(tz) => rrule::Tz::Tz(tz)
                .from_local_datetime(&dt)
                .earliest()
                .and_then(|dt| Some(Ok(dt))),
            Err(err) => Some(Err(err)),
        }
    } else {
        rrule::Tz::Local(chrono::Local)
            .from_local_datetime(&dt)
            .earliest()
            .and_then(|dt| Some(Ok(dt)))
    }
}
