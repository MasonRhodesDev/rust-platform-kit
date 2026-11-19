#[cfg(target_os = "macos")]
mod macos;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Permission {
    Accessibility,
    InputMonitoring,
    PostEvents,
    Microphone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Access {
    Granted,
    Denied,
    Undecided,
    Restricted,
}

impl Permission {
    pub const ALL: [Permission; 4] = [
        Permission::Accessibility,
        Permission::InputMonitoring,
        Permission::PostEvents,
        Permission::Microphone,
    ];

    pub fn settings_pane(self) -> &'static str {
        match self {
            Permission::Accessibility | Permission::PostEvents => "Accessibility",
            Permission::InputMonitoring => "Input Monitoring",
            Permission::Microphone => "Microphone",
        }
    }

    pub fn remedy(self, program: &str) -> String {
        format!(
            "allow {program} in System Settings > Privacy & Security > {}",
            self.settings_pane()
        )
    }
}

#[cfg(target_os = "macos")]
pub fn access(permission: Permission) -> Access {
    macos::access(permission)
}

#[cfg(target_os = "macos")]
pub fn request(permission: Permission) {
    macos::request(permission)
}

#[cfg(not(target_os = "macos"))]
pub fn access(_permission: Permission) -> Access {
    Access::Granted
}

#[cfg(not(target_os = "macos"))]
pub fn request(_permission: Permission) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_posting_is_granted_in_the_accessibility_pane() {
        assert_eq!(Permission::PostEvents.settings_pane(), "Accessibility");
        assert_eq!(
            Permission::InputMonitoring.remedy("/usr/local/bin/app"),
            "allow /usr/local/bin/app in System Settings > Privacy & Security > Input Monitoring"
        );
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn every_permission_is_granted_where_the_os_has_no_privacy_gate() {
        assert!(Permission::ALL
            .iter()
            .all(|p| access(*p) == Access::Granted));
    }
}
