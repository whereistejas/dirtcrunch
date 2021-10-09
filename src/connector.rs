use crate::container::Container;
use async_trait::async_trait;
use futures::{Stream, StreamExt, TryStreamExt};
use serde_json::Value;
use shiplift::Docker;
use std::env;
use std::pin::Pin;
use tokio::fs::{create_dir, File};
use tokio::io::AsyncWriteExt;

/// Core Source trait that defines the Airbyte Connector [specification](https://docs.airbyte.io/understanding-airbyte/airbyte-specification).
#[async_trait]
pub trait Source<'a> {
    /// Name of the source docker image that we are using.
    const IMAGE: &'a str;

    /// This method returns the SPECS for a connector.
    fn specs(&self) -> Value;

    /// Discover the schema of the underlying datasource.
    /// *NOTE*: This method does not handle the parsing of the output recieved from the
    /// discover command. Each connector has it's own output format and its kind of
    /// difficult to handle it in a generic way. We expect the caller to do this on
    /// its own.
    async fn discover<'docker>(&self, config: &Value) -> Vec<String> {
        // Write config to local filesystem, so that it can mounted as a volume.
        if !std::path::Path::new("app/").exists() {
            create_dir("app").await.expect(
                "Failed to create app directory on local filesystem for mounting as a volume.",
            );
        }

        let mut file = File::create("app/config")
            .await
            .expect("Failed to create config file.");
        let config: String = serde_json::to_string(config).unwrap();
        file.write_all(config.as_bytes())
            .await
            .expect("Failed to write JSON config to file.");

        let config_path = format!("{}/app/", env::current_dir().unwrap().to_str().unwrap());
        let path = format!("{}:/app", config_path);

        let docker = Docker::new();
        let mut container = Container::new(&docker);
        container.imagename(Self::IMAGE);

        let read = container
            .start_container(
                vec!["discover", "--config", "/app/config"],
                Some(vec![&path]),
            )
            .await;

        let result_bytes = read
            .try_collect::<Vec<_>>()
            .await
            .expect("Failed to read command output from docker container.");

        container
            .delete_container(true)
            .await
            .expect("Failed to remove container.");

        result_bytes
            .iter()
            .map(|s| String::from_utf8(s.to_vec()).unwrap())
            .collect::<Vec<_>>()
    }

    /// NOTE: To use this method, supply it with a docker object using `Docker::new()`.
    async fn read<'docker>(
        &self,
        docker: &'docker Docker,
        config: &Value,
        catalog: &Value,
    ) -> Pin<Box<dyn Stream<Item = String> + 'docker>> {
        // Write config to local filesystem, so that it can mounted as a volume.
        if !std::path::Path::new("app/").exists() {
            create_dir("app").await.expect(
                "Failed to create app directory on local filesystem for mounting as a volume.",
            );
        }

        let mut file = File::create("app/config")
            .await
            .expect("Failed to create config file.");
        let config: String = serde_json::to_string(config).unwrap();
        file.write_all(config.as_bytes())
            .await
            .expect("Failed to write JSON config to file.");

        let mut file = File::create("app/catalog")
            .await
            .expect("Failed to create config file.");
        let catalog: String = serde_json::to_string(catalog).unwrap();
        file.write_all(catalog.as_bytes())
            .await
            .expect("Failed to write JSON catalog to file.");

        let path = format!("{}/app:/app", env::current_dir().unwrap().to_str().unwrap());

        let mut container = Container::new(docker);
        container.imagename(Self::IMAGE);

        let command = vec![
            "read",
            "--config",
            "/app/config",
            "--catalog",
            "/app/catalog",
        ];

        let volume = vec![path.as_str()];

        let read = container.start_container(command, Some(volume)).await;

        // Convert stream type from to String.
        read.map(|s| String::from_utf8(s.unwrap().to_vec()).unwrap())
            .boxed()
    }
}
