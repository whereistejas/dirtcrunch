// TODO: doing this is probably not the best thing.
use crate::container::Container;
use async_trait::async_trait;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::fs;

/// The set of connector commands that are defined in the specification.
#[derive(Debug, Serialize, Deserialize)]
pub enum AirbyteReturn {
    Config(Value),
    AirbyteConnectionStatus(Value),
    AirbyteCatalog(Value),
    AirbyteStream,
}

/// Core Source trait that defines the Airbyte Connector
/// [specification](https://docs.airbyte.io/understanding-airbyte/airbyte-specification).
#[async_trait]
pub trait Source<'a, Config>
where
    Config: Serialize + Send + Sync,
{
    /// Name of the source docker image that we are using.
    const IMAGE: &'a str;

    /// This method returns the SPECS for a ['CONNECTOR'].
    fn specs(&self) -> AirbyteReturn;

    fn check(&self, config: &Config) -> AirbyteReturn;

    async fn discover(&self, config: &Config) -> AirbyteReturn {
        // Write config to virtual filesystem.
        fs::create_dir("app").await.unwrap();
        fs::write(
            "app/config",
            serde_json::to_string(&config).unwrap().as_bytes(),
        )
        .await
        .unwrap();

        // We need to add volumes to the container.
        let mut container = Container::new();
        container.prepare_image(Self::IMAGE).await;
        let volume = vec!["app/config:app/config"];

        let read = container
            .start_container("discover --config config", Some(volume))
            .await;

        let result_bytes = read
            .try_collect::<Vec<_>>()
            .await
            .expect("Failed to read command output from docker container.");

        let result = result_bytes
            .iter()
            .map(|s| String::from_utf8(s.to_vec()).unwrap())
            .collect::<Vec<_>>();

        println!("{:?}", result);

        AirbyteReturn::AirbyteCatalog(serde_json::from_str(&result[1]).unwrap())
    }

    fn read(&self, config: &Config) -> AirbyteReturn;
}
