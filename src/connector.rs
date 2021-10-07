// TODO: doing this is probably not the best thing.
use crate::{container::Container, core_structs::AirbyteConnectionStatus};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// The set of connector commands that are defined in the specification.
#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Spec(String),
    Check(AirbyteConnectionStatus),
    Discover,
    Read,
}

impl Command {
    /// This method is only applicable for `Command::Spec`.
    pub fn as_str(&self) -> &str {
        match self {
            Command::Spec(spec) => spec,
            _ => "",
        }
    }
}

/// Core Source trait that defines the Airbyte Connector
/// [specification](https://docs.airbyte.io/understanding-airbyte/airbyte-specification).
#[async_trait]
pub trait Source {
    /// Name of the connector for which we are implementing the trait.
    const CONNECTOR: &'static str;

    /// This method returns the SPECS for a ['CONNECTOR'].
    async fn specs(&self) -> Command {
        let mut container = Container::new();
        container.prepare_image(Self::CONNECTOR).await;
        let result = container.start_container("spec").await;

        let search_string = "{\"type\": \"SPEC\",";
        let spec = result
            .iter()
            .find(|s| s.contains(search_string))
            .expect("Could not find specs.");
        let start_index = spec.find(search_string).expect("Could not find specs.");

        let spec = spec.split_at(start_index);

        Command::Spec(spec.1.to_string())
    }
}
