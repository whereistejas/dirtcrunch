// Implements the core Airbyte Source connector trait.
mod connector;
// Handles container operations.
mod container;
// Airbyte data types for various connector commands.
mod core_structs;

pub use connector::Source;
