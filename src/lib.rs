#![deny(
    clippy::all,
//  clippy::restriction,
    clippy::pedantic,
//  clippy::nursery,
//  clippy::cargo,
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::struct_field_names,
    clippy::too_many_lines
)]

/// The financial-accounts application.
pub mod app;
