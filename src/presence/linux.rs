use std::time::{Duration, SystemTime, UNIX_EPOCH};

use logind_session::{is_graphical_type, LogindManagerProxy, LogindSessionProxy};
use zbus::proxy::CacheProperties;
use zbus::OwnedObjectPath;

use super::{backend, Error, Presence};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Seen {
    active: bool,
    locked: bool,
    idle_hint: bool,
    idle_since_usec: u64,
}

pub async fn current(idle_after: Duration) -> Result<Presence, Error> {
    let conn = zbus::Connection::system().await.map_err(backend)?;
    let manager = LogindManagerProxy::new(&conn).await.map_err(backend)?;
    let uid = unsafe { libc::getuid() };
    let mut seen = Vec::new();
    for (_id, owner, _user, _seat, path) in manager.list_sessions().await.map_err(backend)? {
        if owner == uid {
            if let Some(s) = read(&conn, path).await {
                seen.push(s);
            }
        }
    }
    let s = choose(&seen).ok_or(Error::NoSession)?;
    let idle = s.idle_hint && idle_long_enough(s.idle_since_usec, SystemTime::now(), idle_after);
    Ok(Presence::decide(s.locked, !s.active, idle))
}

async fn read(conn: &zbus::Connection, path: OwnedObjectPath) -> Option<Seen> {
    let session = LogindSessionProxy::builder(conn)
        .path(path)
        .cache_properties(CacheProperties::No)
        .build()
        .await
        .ok()?;
    if !is_graphical_type(session.session_type().await.ok()?.as_str()) {
        return None;
    }
    Some(Seen {
        active: session.state().await.ok()? == "active",
        locked: session.locked_hint().await.ok()?,
        idle_hint: session.inner().get_property("IdleHint").await.ok()?,
        idle_since_usec: session.inner().get_property("IdleSinceHint").await.ok()?,
    })
}

fn choose(seen: &[Seen]) -> Option<Seen> {
    seen.iter().find(|s| s.active).or(seen.first()).copied()
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

    fn session(active: bool, locked: bool) -> Seen {
        Seen {
            active,
            locked,
            idle_hint: false,
            idle_since_usec: 0,
        }
    }

    #[test]
    fn the_active_graphical_session_wins_over_earlier_background_ones() {
        let picked = choose(&[session(false, true), session(true, false)]);
        assert_eq!(picked, Some(session(true, false)));
    }

    #[test]
    fn with_no_active_session_the_first_graphical_one_is_reported() {
        assert_eq!(
            choose(&[session(false, true), session(false, false)]),
            Some(session(false, true))
        );
        assert_eq!(choose(&[]), None);
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
