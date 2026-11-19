use std::time::Duration;

use core_foundation::base::{CFType, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::string::CFString;

use super::{Error, Presence};

const COMBINED_SESSION_STATE: i32 = 0;
const ANY_INPUT_EVENT: u32 = u32::MAX;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGSessionCopyCurrentDictionary() -> CFDictionaryRef;
    fn CGEventSourceSecondsSinceLastEventType(state: i32, event_type: u32) -> f64;
}

pub async fn current(idle_after: Duration) -> Result<Presence, Error> {
    let raw = unsafe { CGSessionCopyCurrentDictionary() };
    if raw.is_null() {
        return Err(Error::NoSession);
    }
    let session: CFDictionary<CFString, CFType> =
        unsafe { CFDictionary::wrap_under_create_rule(raw) };
    let flag = |key: &'static str| {
        session
            .find(CFString::from_static_string(key))
            .and_then(|value| value.downcast::<CFBoolean>())
            .is_some_and(bool::from)
    };
    let locked = flag("CGSSessionScreenIsLocked");
    let inactive = !flag("kCGSSessionOnConsoleKey");
    let since_input =
        unsafe { CGEventSourceSecondsSinceLastEventType(COMBINED_SESSION_STATE, ANY_INPUT_EVENT) };
    let idle = Duration::from_secs_f64(since_input.max(0.0)) >= idle_after;
    Ok(Presence::decide(locked, inactive, idle))
}
