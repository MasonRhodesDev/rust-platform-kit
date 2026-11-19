# rust-platform-kit

Two OS facilities that desktop daemons need and that existing crates cover only in part, behind one API on Linux and macOS.

| Feature | What it answers | Linux | macOS | Windows |
|---|---|---|---|---|
| `presence` | Is a person at this machine: `Active`, `Idle`, `Locked`, or `Inactive` (their session is in the background) | logind: the calling user's graphical session, its `LockedHint`, `IdleHint`, `IdleSinceHint` and `State` | `CGSessionCopyCurrentDictionary`, and seconds since the last hardware input event | `Error::Unsupported` |
| `permissions` | Accessibility, Input Monitoring, event posting and microphone access | Always `Granted` | TCC checks and prompts | Always `Granted` |

Both features are on by default. The minimum Rust is 1.89. On macOS the minimum is 10.15.

For needs that existing crates already cover, depend on those crates directly: `etcetera` for paths, `sd-notify` for systemd readiness, `service-manager` for installing a service, and `keepawake` for blocking sleep.

## Usage

```toml
[dependencies]
rust-platform-kit = { git = "https://github.com/MasonRhodesDev/rust-platform-kit", tag = "v0.2.0" }
```

```rust,no_run
use std::time::Duration;

use rust_platform_kit::permissions::{self, Access, Permission};
use rust_platform_kit::presence;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let probe = presence::current(Duration::from_secs(300));
    match tokio::time::timeout(Duration::from_secs(2), probe).await {
        Ok(Ok(now)) => println!("away: {}", now.is_away()),
        Ok(Err(e)) => eprintln!("presence unknown: {e}"),
        Err(_) => eprintln!("presence unknown: timed out"),
    }

    for p in [Permission::Accessibility, Permission::InputMonitoring] {
        match permissions::access(p) {
            Access::Granted => {}
            Access::Restricted => eprintln!("{p:?} is blocked by a device policy"),
            _ => {
                permissions::request(p);
                eprintln!("{}", p.remedy("my-daemon"));
            }
        }
    }
}
```

## Presence

- `current(idle_after)` reports `Idle` once the person has been idle for `idle_after`.
  - On macOS that is time since the last input event the HID system saw. Input a program posts at the HID or session event tap counts too, as a person would; only events posted at the annotated session tap do not. `cargo run --example idle_taps` measures this on the current Mac. A program that injects input has to discount its own events.
  - On Linux the session's idle daemon must set logind's `IdleHint` first, and `idle_after` is then measured from `IdleSinceHint`, so it adds to the desktop's own idle delay. If nothing in the session sets `IdleHint`, Linux never reports `Idle`.
  - An `idle_after` of zero means always `Idle` on macOS, and `Idle` whenever the hint is set on Linux.
- A locked screen is `Locked` before anything else. A session that is not in the foreground (another user, another VT) is `Inactive`.
- On Linux it needs a tokio runtime, and it opens a system bus connection per call, so wrap it in a timeout as above.
- It reads the calling user's sessions only. With no graphical session for that user, it returns `Error::NoSession`, never a guess.

## Permissions

- `access` is a snapshot. `request` raises the system prompt and returns at once; read `access` again later for the answer.
- Accessibility and event posting can only report `Granted` or `Denied`, because macOS does not say whether it has asked, so request them whenever they are not `Granted`. The Accessibility prompt shows on every request. Input Monitoring and the microphone can also report `Undecided`, and only prompt once; after a denial `request` does nothing and the user has to change it in Settings.
- The microphone reports `Restricted` when a device policy blocks it. The user cannot change that in Settings, so `remedy` does not apply to it.
- macOS shows the prompt under the binary's path, so ask from the process that needs the access.
- A microphone prompt needs `NSMicrophoneUsageDescription` in the program's Info.plist. A bare binary can embed one with the linker's `-sectcreate __TEXT __info_plist`. Without it, macOS ends the process when it asks.

`cargo run --example probe` prints what the current machine reports.

## License

MIT
