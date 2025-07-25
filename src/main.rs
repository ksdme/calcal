use anyhow::Context;

mod dbus;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let conn = zbus::connection::Builder::session()
        .context("Could not build dbus session")?
        .build()
        .await
        .context("Could not connect to session dbus")?;

    let sources_proxy = dbus::sources::SourcesObjectManagerProxy::new(&conn)
        .await
        .context("Could not build sources object manager proxy")?;

    let calendar_proxy = dbus::calendar::CalendarFactoryProxy::new(&conn)
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
    println!("{:#?}", calendar_sources);

    for source in calendar_sources.iter() {
        let value = calendar_proxy
            .open_calendar(&source.uid)
            .await
            .context(format!("Could not open calendar for {:?}", source.uid));

        println!("{:?} {:?}", source.uid, value);
    }

    Ok(())
}
