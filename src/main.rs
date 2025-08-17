use std::cell::Cell;

use anyhow::Context;
use calcard::icalendar;
use clap::Parser;
use prettytable::Row;

mod eds;
mod utils;

#[derive(Debug, clap::Parser)]
#[command(about = "Retrieves upcoming events from your calendar.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// List all the calendars available to nexte.
    Calendars,

    /// Generates a summary of all the ongoing events and upcoming events.
    Summary {
        /// The whitelist of calendars to fetch the events from. Defaults to all the calendars
        /// available to nexte. You can run `nexte calendars` to find out the names of all
        /// the available calendars.
        #[arg(short, long)]
        calendars: Option<Vec<String>>,

        /// If enabled, the summary will only contain events from today.
        #[arg(short, long, default_value_t = true)]
        limit_to_today: bool,
    },

    /// Generates a simple table of all the events today.
    Today {
        /// The whitelist of calendars to fetch the events from. Defaults to all the calendars
        /// available to nexte. You can run `nexte calendars` to find out the names of all
        /// the available calendars.
        #[arg(short, long)]
        calendars: Option<Vec<String>>,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let conn = zbus::connection::Builder::session()
        .context("Could not build dbus session")?
        .build()
        .await
        .context("Could not connect to session dbus")?;

    match cli.command {
        Command::Calendars => {
            calendars(&conn).await.context("Could not list calendars")?;
        }

        Command::Summary {
            calendars,
            limit_to_today,
        } => {
            summary(&conn, calendars, limit_to_today)
                .await
                .context("Could not generate summary")?;
        }

        Command::Today { calendars } => {
            today(&conn, calendars)
                .await
                .context("Could not generate full calendar")?;
        }
    }

    Ok(())
}

// Print a list of all the known calendars.
async fn calendars(conn: &zbus::Connection) -> anyhow::Result<()> {
    let calendars = fetch_calendars(conn).await?;

    for cal in calendars.iter() {
        println!(
            "{}",
            match &cal.display_name {
                Some(name) => name,
                None => "Unknown",
            }
        );
    }

    Ok(())
}

// Returns the status of the current or upcoming events.
async fn summary(
    conn: &zbus::Connection,
    whitelist: Option<Vec<String>>,
    limit_to_today: bool,
) -> anyhow::Result<()> {
    let near_events = near_events(conn, whitelist).await?;

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
            utils::human_short_duration(ends.to_utc() - now.to_utc()),
        );
    } else {
        // Remaining events are either in progress or are upcoming.
        if let Some((starts, _, event)) = active_events.first() {
            if !limit_to_today
                || starts.with_timezone(&chrono::Local).date_naive() == now.date_naive()
            {
                print!(
                    "{} in {}",
                    event.title.clone().unwrap_or("Unknown Event".to_owned()),
                    utils::human_short_duration(starts.to_utc() - now.to_utc()),
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

// Prints a list of all the events today.
async fn today(conn: &zbus::Connection, whitelist: Option<Vec<String>>) -> anyhow::Result<()> {
    let near_events = near_events(conn, whitelist).await?;

    // Filter for today.
    let today = chrono::Local::now().date_naive();
    let today_events = near_events
        .into_iter()
        .filter(|e| {
            if let Some(starts) = e.starts {
                starts.with_timezone(&chrono::Local).date_naive() == today
            } else {
                false
            }
        })
        .collect::<Vec<eds::event::Event>>();

    if today_events.len() == 0 {
        println!("No Events Today");
        return Ok(());
    }

    // Put them in a table.
    let mut table = prettytable::Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_CLEAN);

    for item in today_events.iter() {
        let starts = item
            .starts
            .map(|dt| utils::human_short_time(dt))
            .unwrap_or("?".to_owned());

        let ends = item
            .ends
            .map(|dt| utils::human_short_time(dt))
            .unwrap_or("?".to_owned());

        table.add_row(prettytable::row![
            item.title.as_deref().unwrap_or("Unknown Event"),
            &format!("{}-{}", starts, ends),
        ]);
    }
    table.printstd();

    Ok(())
}

// Returns a list of near events.
async fn near_events(
    conn: &zbus::Connection,
    whitelist: Option<Vec<String>>,
) -> anyhow::Result<Vec<eds::event::Event>> {
    let mut calendars = fetch_calendars(conn).await?;

    // Apply the whitelist if necessary.
    if let Some(whitelist) = whitelist {
        calendars = calendars
            .into_iter()
            .filter(|c| match &c.display_name {
                Some(name) => whitelist.contains(name),
                _ => false,
            })
            .collect();
    }

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
            _ => true,
        })
        .collect();

    // Sort all events by start time.
    near_events.sort_by(|a, b| a.starts.cmp(&b.starts));

    Ok(near_events)
}

// Return a list of calendars from the connection.
async fn fetch_calendars(conn: &zbus::Connection) -> anyhow::Result<Vec<eds::calendar::Calendar>> {
    let mut calendars = eds::calendar::Calendar::fetch_all(&conn)
        .await
        .context("Could not list all calendars")?;

    // Sort them so you have a stable order.
    calendars.sort_by(|a, b| a.display_name.cmp(&b.display_name));

    Ok(calendars)
}
