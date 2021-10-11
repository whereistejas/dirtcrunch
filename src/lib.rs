// Implements the core Airbyte Source connector trait.
mod connector;
// Handles container operations.
mod container;
// Utility methods, that are used in `build.rs` to generate the `Source` trait implementations.
mod util;

pub use connector::Source;
pub use shiplift::Docker;
pub use util::source::create_file;
