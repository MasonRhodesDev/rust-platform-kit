use std::time::Duration;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux as imp;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as imp;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Presence {
    Active,
    Idle,
    Locked,
    Inactive,
}

impl Presence {
    #[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
    pub(crate) fn decide(locked: bool, inactive: bool, idle: bool) -> Self {
        if locked {
            Presence::Locked
        } else if inactive {
            Presence::Inactive
        } else if idle {
            Presence::Idle
        } else {
            Presence::Active
        }
    }

    pub fn is_away(self) -> bool {
        self != Presence::Active
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("presence is not implemented on this platform")]
    Unsupported,
    #[error("no graphical session for this user")]
    NoSession,
    #[error("{0}")]
    Backend(Box<dyn std::error::Error + Send + Sync>),
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn backend(e: impl std::error::Error + Send + Sync + 'static) -> Error {
    Error::Backend(Box::new(e))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub async fn current(idle_after: Duration) -> Result<Presence, Error> {
    imp::current(idle_after).await
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub async fn current(_idle_after: Duration) -> Result<Presence, Error> {
    Err(Error::Unsupported)
}

#[cfg(test)]
mod tests {
    use super::Presence;

    #[test]
    fn a_locked_screen_outranks_every_other_signal() {
        assert_eq!(Presence::decide(true, true, true), Presence::Locked);
        assert_eq!(Presence::decide(true, false, false), Presence::Locked);
    }

    #[test]
    fn a_session_in_the_background_is_inactive_before_it_is_idle() {
        assert_eq!(Presence::decide(false, true, true), Presence::Inactive);
        assert_eq!(Presence::decide(false, false, true), Presence::Idle);
    }

    #[test]
    fn only_an_unlocked_foreground_session_in_use_is_active() {
        assert_eq!(Presence::decide(false, false, false), Presence::Active);
        assert!(!Presence::Active.is_away());
        assert!(Presence::Idle.is_away());
        assert!(Presence::Locked.is_away());
        assert!(Presence::Inactive.is_away());
    }
}
