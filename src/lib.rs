//! Components for building a status bar.

#![forbid(unsafe_code)]
#![warn(
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
#![allow(clippy::module_name_repetitions, clippy::multiple_crate_versions)]

pub mod bar;
pub mod block;
pub mod config;
pub mod dbus;
pub mod format;
pub mod modules;
