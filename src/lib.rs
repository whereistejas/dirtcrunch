// Implements the core Airbyte Source connector trait.
mod connector;
// Handles container operations.
mod container;
// Utility methods.
mod util;

pub use connector::{AirbyteReturn, Source};
pub use util::{container::*, file::*};
