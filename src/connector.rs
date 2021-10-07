// TODO: doing this is probably not the best thing.
use crate::core_structs::AirbyteConnectionStatus;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The set of connector commands that are defined in the specification.
#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Spec(Value),
    Check(AirbyteConnectionStatus),
    Discover,
    Read,
}

/// Core Source trait that defines the Airbyte Connector
/// [specification](https://docs.airbyte.io/understanding-airbyte/airbyte-specification).
#[async_trait]
pub trait Source<Config> {
    /// This method returns the SPECS for a ['CONNECTOR'].
    fn specs(&self) -> Command;
    async fn discover(&self, config: &Config) -> Command;
    fn read(&self, config: &Config) -> Command;
}
