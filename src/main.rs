use anyhow::Context;

mod dbus;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let conn = zbus::connection::Builder::session()
        .context("Could not build dbus session")?
        .build()
        .await
        .context("Could not connect to session dbus")?;

    let sources_proxy = dbus::SourcesObjectManagerProxy::new(&conn)
        .await
        .context("Could not build Sources object manager proxy")?;

    let sources = sources_proxy
        .list_sources()
        .await
        .context("Could not query for managed objects")?;

    println!("{:#?}", sources);

    Ok(())
}
