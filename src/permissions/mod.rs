#[cfg(target_os = "macos")]
mod macos;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    Accessibility,
    InputMonitoring,
    PostEvents,
    Microphone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    Granted,
    Denied,
    Undecided,
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
pub fn request(permission: Permission) -> Access {
    macos::request(permission)
}

#[cfg(not(target_os = "macos"))]
pub fn access(_permission: Permission) -> Access {
    Access::Granted
}

#[cfg(not(target_os = "macos"))]
pub fn request(_permission: Permission) -> Access {
    Access::Granted
}

pub fn missing(permissions: &[Permission]) -> Vec<Permission> {
    permissions
        .iter()
        .copied()
        .filter(|p| access(*p) != Access::Granted)
        .collect()
}

#[cfg(any(target_os = "macos", test))]
fn from_hid_access(value: u32) -> Access {
    match value {
        0 => Access::Granted,
        1 => Access::Denied,
        _ => Access::Undecided,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hid_access_maps_granted_denied_and_unknown() {
        assert_eq!(from_hid_access(0), Access::Granted);
        assert_eq!(from_hid_access(1), Access::Denied);
        assert_eq!(from_hid_access(2), Access::Undecided);
    }

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
    fn nothing_is_missing_where_the_os_has_no_privacy_gate() {
        assert!(missing(&Permission::ALL).is_empty());
    }
}
