use anyhow::Context;
use calcard::icalendar;

#[derive(Debug)]
pub struct Event {
    pub uid: Option<String>,

    pub title: Option<String>,
    pub description: Option<String>,

    pub status: Option<icalendar::ICalendarStatus>,
    pub starts: Option<jiff::Zoned>,
    pub ends: Option<jiff::Zoned>,
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
            starts: zoned_calendar_property(&component, &icalendar::ICalendarProperty::Dtstart)
                .and_then(|dt| dt.ok()),
            ends: zoned_calendar_property(&component, &icalendar::ICalendarProperty::Dtend)
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
fn zoned_calendar_property(
    component: &icalendar::ICalendarComponent,
    property: &icalendar::ICalendarProperty,
) -> Option<anyhow::Result<jiff::Zoned>> {
    let property = component.property(property)?;

    let tz = property.params.first().and_then(|value| match value {
        icalendar::ICalendarParameter::Tzid(tz) => Some(tz),
        _ => None,
    });
    let tz = if let Some(tz) = tz {
        match jiff::tz::TimeZone::get(tz).context("Could not resolve timezone") {
            Ok(tz) => tz,
            Err(err) => return Some(Err(err)),
        }
    } else {
        jiff::tz::TimeZone::system()
    };

    let dt = property.values.first()?.as_partial_date_time()?;
    let now = jiff::Zoned::now();
    Some(
        jiff::civil::DateTime::new(
            dt.year.and_then(|y| Some(y as i16)).unwrap_or(now.year()),
            dt.month.and_then(|m| Some(m as i8)).unwrap_or(now.month()),
            dt.day.and_then(|d| Some(d as i8)).unwrap_or(now.day()),
            dt.hour.and_then(|h| Some(h as i8)).unwrap_or_default(),
            dt.minute.and_then(|m| Some(m as i8)).unwrap_or_default(),
            dt.second.and_then(|s| Some(s as i8)).unwrap_or_default(),
            0,
        )
        .context("Could not build civil date time")
        .and_then(|dt| {
            dt.to_zoned(tz)
                .context("Could not build timezone aware date time")
        }),
    )
}
