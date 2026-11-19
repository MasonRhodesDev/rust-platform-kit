use block2::RcBlock;
use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::string::{CFString, CFStringRef};
use objc2::runtime::Bool;
use objc2_av_foundation::{AVAuthorizationStatus, AVCaptureDevice, AVMediaTypeAudio};

use super::{Access, Permission};

const LISTEN_EVENT: u32 = 1;

#[allow(non_upper_case_globals)]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    static kAXTrustedCheckOptionPrompt: CFStringRef;
    fn AXIsProcessTrusted() -> u8;
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> u8;
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPreflightPostEventAccess() -> bool;
    fn CGRequestPostEventAccess() -> bool;
}

#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOHIDCheckAccess(request: u32) -> u32;
    fn IOHIDRequestAccess(request: u32) -> bool;
}

fn from_hid_access(value: u32) -> Access {
    match value {
        0 => Access::Granted,
        1 => Access::Denied,
        _ => Access::Undecided,
    }
}

fn granted(yes: bool) -> Access {
    if yes {
        Access::Granted
    } else {
        Access::Denied
    }
}

fn microphone() -> Access {
    let Some(audio) = (unsafe { AVMediaTypeAudio }) else {
        return Access::Undecided;
    };
    let status = unsafe { AVCaptureDevice::authorizationStatusForMediaType(audio) };
    if status == AVAuthorizationStatus::Authorized {
        Access::Granted
    } else if status == AVAuthorizationStatus::NotDetermined {
        Access::Undecided
    } else if status == AVAuthorizationStatus::Restricted {
        Access::Restricted
    } else {
        Access::Denied
    }
}

pub fn access(permission: Permission) -> Access {
    match permission {
        Permission::Accessibility => granted(unsafe { AXIsProcessTrusted() } != 0),
        Permission::InputMonitoring => from_hid_access(unsafe { IOHIDCheckAccess(LISTEN_EVENT) }),
        Permission::PostEvents => granted(unsafe { CGPreflightPostEventAccess() }),
        Permission::Microphone => microphone(),
    }
}

pub fn request(permission: Permission) {
    match permission {
        Permission::Accessibility => {
            let prompt = unsafe { CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt) };
            let options = CFDictionary::from_CFType_pairs(&[(prompt, CFBoolean::true_value())]);
            unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) };
        }
        Permission::InputMonitoring => {
            unsafe { IOHIDRequestAccess(LISTEN_EVENT) };
        }
        Permission::PostEvents => {
            unsafe { CGRequestPostEventAccess() };
        }
        Permission::Microphone => {
            if microphone() == Access::Undecided {
                if let Some(audio) = unsafe { AVMediaTypeAudio } {
                    let answered = RcBlock::new(|_granted: Bool| {});
                    unsafe {
                        AVCaptureDevice::requestAccessForMediaType_completionHandler(
                            audio, &answered,
                        )
                    };
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hid_access_follows_iohid_access_type() {
        assert_eq!(from_hid_access(0), Access::Granted);
        assert_eq!(from_hid_access(1), Access::Denied);
        assert_eq!(from_hid_access(2), Access::Undecided);
    }
}
