use crate::container::Container;
use async_trait::async_trait;
use futures::{Stream, StreamExt, TryStreamExt};
use serde_json::Value;
use shiplift::Docker;
use std::{env, pin::Pin};
use tokio::{
    fs::{create_dir, File},
    io::AsyncWriteExt,
};

/// This is the core trait that defines the [Airbyte connector specification](https://docs.airbyte.io/understanding-airbyte/airbyte-specification).
#[async_trait]
pub trait Source<'a> {
    /// Name of the source docker image that we are using.
    const IMAGE: &'a str;

    /// This method returns the SPECS for a connector.
    fn specs(&self) -> Value;

    /// Discover the schema of the underlying datasource.
    /// NOTE: This method doesn't parse the output received from the `discover` command. The caller
    /// is expected to extracted the useful information from the returned `String`, on their own.
    async fn discover<'docker>(&self, config: &Value) -> Vec<String> {
        // Check if the `app` folder exists. We will bind this folder to the container as a volume.
        if !std::path::Path::new("app/").exists() {
            create_dir("app").await.expect(
                "Failed to create app directory on local filesystem for mounting as a volume.",
            );
        }

        // Create the config file.
        let mut file = File::create("app/config")
            .await
            .expect("Failed to create config file.");

        // Write the config JSON to the file.
        let config: String = serde_json::to_string(config).unwrap();
        file.write_all(config.as_bytes())
            .await
            .expect("Failed to write JSON config to file.");

        // Create the paths to the `app` folder that will be used for binding the folder to the
        // container as a volume.
        let path = format!(
            "{}/app/:/app",
            env::current_dir().unwrap().to_str().unwrap()
        );

        let docker = Docker::new();
        let mut container = Container::new(&docker);
        container.imagename(Self::IMAGE);

        // Create container, run the command and receive a stream that will be used to read from
        // the stdout of the container.
        let read = container
            .start_container(
                vec!["discover", "--config", "/app/config"],
                Some(vec![&path]),
            )
            .await;

        // Read command output as a byte vector.
        let result_bytes = read
            .try_collect::<Vec<_>>()
            .await
            .expect("Failed to read command output from docker container.");

        // Remove the container and volumes.
        container
            .delete_container(true)
            .await
            .expect("Failed to remove container.");

        // Convert byte vectors to UTF-8 character strings.
        result_bytes
            .iter()
            .map(|s| String::from_utf8(s.to_vec()).unwrap())
            .collect::<Vec<_>>()
    }

    /// Read data from the connector source, based on the schema recieved from the `Discover`
    /// command.
    /// NOTE: This method returns a Stream which will have to awaited to recieve data from the
    /// source.
    async fn read<'docker>(
        &self,
        docker: &'docker Docker,
        config: &Value,
        catalog: &Value,
    ) -> Pin<Box<dyn Stream<Item = String> + 'docker>> {
        // Check if the `app` folder exists. We will bind this folder to the container as a volume.
        if !std::path::Path::new("app/").exists() {
            create_dir("app").await.expect(
                "Failed to create app directory on local filesystem for mounting as a volume.",
            );
        }

        // Create the config file.
        let mut file = File::create("app/config")
            .await
            .expect("Failed to create config file.");

        // Write the config JSON to the file.
        let config: String = serde_json::to_string(config).unwrap();
        file.write_all(config.as_bytes())
            .await
            .expect("Failed to write JSON config to file.");

        // Create the catalog file.
        let mut file = File::create("app/catalog")
            .await
            .expect("Failed to create config file.");

        // Write the catalog JSON to the file.
        let catalog: String = serde_json::to_string(catalog).unwrap();
        file.write_all(catalog.as_bytes())
            .await
            .expect("Failed to write JSON catalog to file.");

        // Create the paths to the `app` folder that will be used for binding the folder to the
        // container as a volume.
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

        // Create container, run the command and receive a stream that will be used to read from
        // the stdout of the container.
        let read = container.start_container(command, Some(volume)).await;

        // Convert the type of the `read` stream from `&[u8]` to `String`.
        read.map(|s| String::from_utf8(s.unwrap().to_vec()).unwrap())
            .boxed()
    }
}
