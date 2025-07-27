use anyhow::Context;
use calcard::icalendar;
use chrono::{Datelike, TimeZone};
use chrono_tz::Tz;
use chrono_tz::UTC;

#[derive(Debug)]
pub struct Event {
    pub uid: Option<String>,

    pub title: Option<String>,
    pub description: Option<String>,

    pub status: Option<icalendar::ICalendarStatus>,
    pub starts: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub ends: Option<chrono::DateTime<chrono::FixedOffset>>,
}

impl From<icalendar::ICalendarComponent> for Event {
    fn from(component: icalendar::ICalendarComponent) -> Self {
        Self {
            uid: component.uid().map(|uid| uid.to_owned()),

            title: str_calendar_property(&component, &icalendar::ICalendarProperty::Summary),
            description: str_calendar_property(
                &component,
                &icalendar::ICalendarProperty::Description,
            ),

            status: component.status().map(|status| status.clone()),
            starts: fixed_offset_datetime_calendar_property(
                &component,
                &icalendar::ICalendarProperty::Dtstart,
            )
            .and_then(|dt| dt.ok()),
            ends: fixed_offset_datetime_calendar_property(
                &component,
                &icalendar::ICalendarProperty::Dtend,
            )
            .and_then(|dt| dt.ok()),
        }
    }
}

// Load up a property from the calendar component as a string value.
fn str_calendar_property(
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
fn fixed_offset_datetime_calendar_property(
    component: &icalendar::ICalendarComponent,
    property: &icalendar::ICalendarProperty,
) -> Option<anyhow::Result<chrono::DateTime<chrono::FixedOffset>>> {
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
            Ok(tz) => tz
                .from_local_datetime(&dt)
                .earliest()
                .map(|dt| dt.fixed_offset())
                .and_then(|dt| Some(Ok(dt))),
            Err(err) => Some(Err(err)),
        }
    } else {
        chrono::Local
            .from_local_datetime(&dt)
            .earliest()
            .map(|dt| dt.fixed_offset())
            .and_then(|dt| Some(Ok(dt)))
    }
}
