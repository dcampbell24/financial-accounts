#![deny(
    clippy::all,
//  clippy::restriction,
    clippy::pedantic,
//  clippy::nursery,
    clippy::cargo,
)]
#![allow(
    clippy::similar_names,
    clippy::struct_field_names,
    clippy::multiple_crate_versions
)]

/// The financial-accounts application.
pub mod app;
