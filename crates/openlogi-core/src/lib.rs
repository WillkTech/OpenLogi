//! Shared types and configuration for OpenLogi.
//!
//! Everything here is data — the device model, the action catalogue, the
//! binding types, the shape of the config file. It must never depend on
//! `hidpp`, `async-hid`, or any platform-specific event/window API; those live
//! in sibling crates.
//!
//! The one exception is reading and writing that config file, which is gated
//! behind the `fs` feature (on by default). Without it this crate touches no
//! host at all, which is what the `wasm (portable crates)` CI job checks.

#![deny(missing_docs)]
#[cfg(test)]
#[macro_use]
mod test_macros {
    macro_rules! assert_matches {
        ($expression:expr, $pattern:pat $(if $guard:expr)? $(,)?) => {
            match $expression {
                $pattern $(if $guard)? => (),
                ref e => panic!("assertion failed: `(left matches right)`\n  left: `{:?}`\n right: `{}`", e, stringify!($pattern)),
            }
        };
    }
}

pub mod action_ring;
pub mod app;
pub mod binding;
pub mod bindings;
pub mod brand;
pub mod color;
pub mod config;
pub mod device;
pub mod device_order;
pub mod diagnostics;
pub mod hid;
pub mod math;
#[cfg(feature = "fs")]
pub mod paths;
pub mod scroll;
#[cfg(feature = "fs")]
pub mod single_instance;
