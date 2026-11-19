#[cfg(target_os = "macos")]
fn main() {
    use std::ffi::c_void;
    use std::time::Duration;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceSecondsSinceLastEventType(state: i32, event_type: u32) -> f64;
        fn CGEventCreateKeyboardEvent(source: *const c_void, key: u16, down: bool) -> *mut c_void;
        fn CGEventPost(tap: u32, event: *mut c_void);
        fn CGPreflightPostEventAccess() -> bool;
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRelease(cf: *const c_void);
    }

    const HID_SYSTEM_STATE: i32 = 1;
    const ANY_INPUT_EVENT: u32 = u32::MAX;
    const SHIFT: u16 = 56;

    let idle =
        || unsafe { CGEventSourceSecondsSinceLastEventType(HID_SYSTEM_STATE, ANY_INPUT_EVENT) };
    let post = |tap: u32| unsafe {
        for down in [true, false] {
            let event = CGEventCreateKeyboardEvent(std::ptr::null(), SHIFT, down);
            if !event.is_null() {
                CGEventPost(tap, event);
                CFRelease(event);
            }
        }
    };

    if !unsafe { CGPreflightPostEventAccess() } {
        println!("this process may not post events; grant it Accessibility and run again");
        return;
    }
    println!("leave the keyboard and mouse alone for the next 12 seconds");
    for (name, tap) in [("session", 1u32), ("annotated-session", 2), ("hid", 0)] {
        std::thread::sleep(Duration::from_secs(3));
        let before = idle();
        post(tap);
        std::thread::sleep(Duration::from_millis(500));
        let after = idle();
        println!(
            "{name} tap: idle {before:.2}s -> {after:.2}s, resets: {}",
            after < before - 1.0
        );
    }
}

#[cfg(not(target_os = "macos"))]
fn main() {
    println!("macOS only");
}
