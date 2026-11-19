use std::time::Duration;

use super::{Error, Presence};

pub async fn current(_idle_after: Duration) -> Result<Presence, Error> {
    let conn = zbus::Connection::system().await?;
    let session = logind_session::resolve_graphical_session(&conn).await?;
    let proxy = session.proxy(&conn).await?;
    let locked = proxy.locked_hint().await?;
    let inactive = proxy.state().await? != "active";
    let idle: bool = proxy.inner().get_property("IdleHint").await?;
    Ok(Presence::decide(locked, inactive, idle))
}
