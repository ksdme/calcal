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
        println!("{:#?}", calendar.display_name);

        let events = calendar
            .fetch_today_events(&conn)
            .await
            .context("Could not fetch today events")?;

        println!("{:#?}", events);
    }

    Ok(())
}
