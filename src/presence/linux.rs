use std::time::{Duration, SystemTime, UNIX_EPOCH};

use logind_session::{is_graphical_type, LogindManagerProxy, LogindSessionProxy};

use super::{backend, Error, Presence};

pub async fn current(idle_after: Duration) -> Result<Presence, Error> {
    let conn = zbus::Connection::system().await.map_err(backend)?;
    let manager = LogindManagerProxy::new(&conn).await.map_err(backend)?;
    let uid = unsafe { libc::getuid() };
    let mut chosen: Option<(bool, LogindSessionProxy<'_>)> = None;
    for (_id, owner, _user, _seat, path) in manager.list_sessions().await.map_err(backend)? {
        if owner != uid {
            continue;
        }
        let Ok(session) = LogindSessionProxy::builder(&conn).path(path).build().await else {
            continue;
        };
        if !session
            .session_type()
            .await
            .is_ok_and(|t| is_graphical_type(t.as_str()))
        {
            continue;
        }
        let active = session.state().await.is_ok_and(|s| s == "active");
        if active {
            chosen = Some((true, session));
            break;
        }
        if chosen.is_none() {
            chosen = Some((false, session));
        }
    }
    let Some((active, session)) = chosen else {
        return Err(Error::NoSession);
    };
    let locked = session.locked_hint().await.map_err(backend)?;
    let hint: bool = session
        .inner()
        .get_property("IdleHint")
        .await
        .map_err(backend)?;
    let since: u64 = session
        .inner()
        .get_property("IdleSinceHint")
        .await
        .map_err(backend)?;
    let idle = hint && idle_long_enough(since, SystemTime::now(), idle_after);
    Ok(Presence::decide(locked, !active, idle))
}

fn idle_long_enough(since_usec: u64, now: SystemTime, idle_after: Duration) -> bool {
    if since_usec == 0 {
        return true;
    }
    let since = UNIX_EPOCH + Duration::from_micros(since_usec);
    now.duration_since(since).unwrap_or(Duration::ZERO) >= idle_after
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(secs: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(secs)
    }

    #[test]
    fn the_session_hint_counts_only_once_it_is_older_than_the_threshold() {
        let went_idle = 1_000 * 1_000_000;
        assert!(!idle_long_enough(
            went_idle,
            at(1_059),
            Duration::from_secs(60)
        ));
        assert!(idle_long_enough(
            went_idle,
            at(1_060),
            Duration::from_secs(60)
        ));
    }

    #[test]
    fn a_hint_without_a_timestamp_is_taken_as_idle() {
        assert!(idle_long_enough(0, at(5), Duration::from_secs(3600)));
    }

    #[test]
    fn a_timestamp_from_the_future_reads_as_just_gone_idle() {
        assert!(!idle_long_enough(
            2_000 * 1_000_000,
            at(1_000),
            Duration::from_secs(1)
        ));
        assert!(idle_long_enough(
            2_000 * 1_000_000,
            at(1_000),
            Duration::ZERO
        ));
    }
}
