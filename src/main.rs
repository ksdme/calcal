use anyhow::Context;

mod eds;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let conn = zbus::connection::Builder::session()
        .context("Could not build dbus session")?
        .build()
        .await
        .context("Could not connect to session dbus")?;

    let calendars = eds::calendar::Calendar::fetch_all(&conn)
        .await
        .context("Could not list all calendars")?;

    for calendar in calendars.iter() {
        let events = calendar
            .fetch_near_events()
            .await
            .context("Could not fetch today events")?;

        println!("{:?} {:#?}", calendar.display_name, events);
    }

    Ok(())
}
