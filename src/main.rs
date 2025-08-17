use anyhow::Context;
use calcard::icalendar;

mod eds;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let conn = zbus::connection::Builder::session()
        .context("Could not build dbus session")?
        .build()
        .await
        .context("Could not connect to session dbus")?;

    let mut calendars = eds::calendar::Calendar::fetch_all(&conn)
        .await
        .context("Could not list all calendars")?;
    calendars.sort_by(|a, b| a.display_name.cmp(&b.display_name));

    let mut near_events = Vec::new();
    for calendar in calendars.iter() {
        let mut events = calendar
            .fetch_near_events()
            .await
            .context("Could not fetch today events")?;
        near_events.append(&mut events);
    }

    // Remove all events that are not happening.
    // TODO: Ideally, we should check attendees and remove events that you declined.
    near_events = near_events
        .into_iter()
        .filter(|e| match e.status {
            Some(icalendar::ICalendarStatus::Tentative) => true,
            Some(icalendar::ICalendarStatus::Confirmed) => true,
            Some(icalendar::ICalendarStatus::Completed) => true,
            Some(icalendar::ICalendarStatus::Final) => true,
            Some(icalendar::ICalendarStatus::InProcess) => true,
            _ => false,
        })
        .collect();

    // Sort all events by start time.
    near_events.sort_by(|a, b| a.starts.cmp(&b.starts));

    // Filter out events that do not have a start and an end date.
    // Filter out events that were completed in the past.
    let now = chrono::Local::now().with_timezone(&rrule::Tz::Local(chrono::Local));
    let active_events = near_events
        .iter()
        .filter_map(|event| match (event.starts, event.ends) {
            (Some(starts), Some(ends)) if ends > now => Some((starts, ends, event)),
            _ => None,
        })
        .collect::<Vec<(
            chrono::DateTime<rrule::Tz>,
            chrono::DateTime<rrule::Tz>,
            &eds::event::Event,
        )>>();

    // Check if there are any in progress events.
    if let Some((_, ends, event)) = active_events
        .iter()
        .filter(|(starts, ends, _)| starts <= &now && ends > &now)
        .nth(0)
    {
        println!(
            "{} ends in {}",
            event.title.clone().unwrap_or("Unknown Event".to_owned()),
            report_duration(ends.to_utc() - now.to_utc()),
        );
    } else {
        // Remaining events are either in progress or are upcoming.
        if let Some((starts, _, event)) = active_events.first() {
            if starts.with_timezone(&chrono::Local).date_naive() == now.date_naive() {
                print!(
                    "{} in {}",
                    event.title.clone().unwrap_or("Unknown Event".to_owned()),
                    report_duration(starts.to_utc() - now.to_utc()),
                );
            } else {
                print!("No Upcoming Event Today");
            }
        } else {
            println!("No Upcoming Event");
        }
    }

    Ok(())
}

// Returns a short human formatted duration.
fn report_duration(delta: chrono::TimeDelta) -> String {
    let mut seconds = delta.num_seconds().max(0) as u64;
    seconds -= seconds % 60;

    let duration = std::time::Duration::new(seconds, 0);
    humantime::format_duration(duration).to_string()
}
