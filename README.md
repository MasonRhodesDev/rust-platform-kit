# rust-platform-kit

OS facilities that desktop daemons need and that no maintained crate covers, behind one API on Linux and macOS.

| Feature | What it answers | Linux | macOS | Windows |
|---|---|---|---|---|
| `presence` | Is a person at this machine: active, idle, locked, or in a background session | logind `LockedHint`, `IdleHint` and `State` of the graphical session | `CGSessionCopyCurrentDictionary` and seconds since the last input event | `Error::Unsupported` |
| `permissions` | Has the user granted Accessibility, Input Monitoring, event posting and the microphone | Always granted | TCC checks and prompts | Always granted |

Both features are on by default.

For needs that existing crates already cover, depend on those crates directly: `etcetera` for paths, `sd-notify` for systemd readiness, `service-manager` for installing a service, and `keepawake` for blocking sleep.

## Usage

```toml
[dependencies]
rust-platform-kit = { git = "https://github.com/MasonRhodesDev/rust-platform-kit", tag = "v0.1.0" }
```

```rust
use std::time::Duration;
use rust_platform_kit::permissions::{self, Permission};
use rust_platform_kit::presence::{self, Presence};

let now = presence::current(Duration::from_secs(60)).await?;
if now.is_away() { /* ... */ }

for p in permissions::missing(&[Permission::Accessibility, Permission::InputMonitoring]) {
    eprintln!("{}", p.remedy("my-daemon"));
}
```

The Linux `presence` backend uses zbus on tokio, so call it from a tokio runtime. Linux leaves the idle decision to the session's idle daemon (the one that sets logind's `IdleHint`), so `idle_after` applies on macOS only. A machine with no graphical session gets an error, never a guess.

`cargo run --example probe` prints what the current machine reports.

## License

MIT
