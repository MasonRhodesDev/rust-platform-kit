#![cfg_attr(
    all(feature = "presence", feature = "permissions"),
    doc = include_str!("../README.md")
)]

#[cfg(feature = "permissions")]
pub mod permissions;
#[cfg(feature = "presence")]
pub mod presence;
