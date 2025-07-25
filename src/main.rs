use anyhow::Context;

mod dbus;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let conn = zbus::connection::Builder::session()
        .context("Could not build dbus session")?
        .build()
        .await
        .context("Could not connect to session dbus")?;

    let sources_proxy = dbus::sources::SourcesProxy::new(&conn)
        .await
        .context("Could not build sources object manager proxy")?;

    let calendar_factory_proxy = dbus::calendar::CalendarFactoryProxy::new(&conn)
        .await
        .context("Could not build calendar factory proxy")?;

    let sources = sources_proxy
        .list_sources()
        .await
        .context("Could not query for managed objects")?;

    let calendar_sources: Vec<&dbus::sources::Source> = sources
        .iter()
        .filter(|source| source.has_calendar)
        .collect();

    for source in calendar_sources.iter() {
        let (path, _) = calendar_factory_proxy
            .open_calendar(&source.uid)
            .await
            .context(format!("Could not open calendar for {:?}", source.uid))?;

        let calendar_proxy = dbus::calendar::CalendarProxy::builder(&conn)
            .path(path)
            .context("Could not set path on calendar proxy")?
            .build()
            .await
            .context("Could not create calendar proxy")?;

        let events = calendar_proxy
            .list_today_events()
            .await
            .context("Could not load today events")?;
    }

    Ok(())
}
