#![warn(mismatched_lifetime_syntaxes)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod build;
pub mod error;
pub mod repo;
pub mod schema;
pub mod store;

pub use error::{Error, Result};
pub use schema::kdl::Package;
