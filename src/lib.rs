// Implements the core Airbyte Source connector trait.
mod connector;
// Handles container operations.
mod container;
// Utility methods.
mod util;

pub use connector::Source;
pub use util::{container::*, file::*};
