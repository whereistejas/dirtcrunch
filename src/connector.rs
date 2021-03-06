use crate::container::Container;
use anyhow::Result;
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use regex::Regex;
use serde_json::Value;
use shiplift::Docker;
use std::{env, path::Path, pin::Pin};
use tokio::fs::{self, create_dir, File};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio_util::io::StreamReader;

/// This is the core trait that defines the [Airbyte connector specification](https://docs.airbyte.io/understanding-airbyte/airbyte-specification).
#[async_trait]
pub trait Source {
    /// Name of the source docker image that we are using.
    const IMAGE: &'static str;

    /// This method returns the SPECS for a connector.
    fn specs(&self) -> Value;

    /// Discover the schema of the underlying datasource.
    /// This method returns the CATALOG json object as a String.
    async fn discover(&self, config: &Value) -> String {
        // Set path for the `app` folder.
        let app_path = format!("{}/app", env::current_dir().unwrap().to_str().unwrap());

        create_app_folder(&app_path)
            .await
            .expect("failed to create app directory on local filesystem for mounting as a volume.");

        let config_path = format!("{}/config", app_path);
        write_file(&config_path, config).await;

        let docker = Docker::new();
        let mut container = Container::new(&docker);
        container.image_name(Self::IMAGE);

        // Create the paths to the `app` folder that will be used for binding the folder to the
        // container as a volume.
        let volume_path = format!("{}:/app/", app_path);

        // Create container, run the command and receive a stream that will be used to read from
        // the stdout of the container.
        let stream = container
            .start_container(
                vec!["discover", "--config", "/app/config"],
                Some(vec![&volume_path]),
            )
            .await;

        // Convert `Stream` to `StreamReader` object. This allows us to use utilities from `AsyncBufReadNext` on the stream.
        let mut reader = StreamReader::new(stream);

        let mut catalog = String::new();
        while let Ok(result) = reader.read_line(&mut catalog).await {
            if result != 0 {
                let regex = Regex::new(r#"\{"type"\s*:\s*"CATALOG"\s*,"#)
                    .expect("Unable to compile given regular expression.");

                if regex.is_match(catalog.as_str()) {
                    break;
                } else {
                    catalog.clear();
                }
            } else {
                panic!("Could not find CATALOG object.")
            }
        }

        // Remove the container and volumes.
        container.delete_container(true).await;

        // Remove config file.
        remove_file(&config_path).await;

        catalog
    }

    /// Read data from the connector source, based on the schema received from the `Discover`
    /// command.
    /// NOTE: This method returns a Stream which will have to awaited to receive data from the
    /// source.
    async fn read<'docker>(
        &self,
        docker: &'docker Docker,
        config: &Value,
        catalog: &Value,
    ) -> Pin<Box<dyn Stream<Item = String> + 'docker>> {
        // Set path for the `app` folder.
        let app_path = format!("{}/app", env::current_dir().unwrap().to_str().unwrap());

        create_app_folder(&app_path)
            .await
            .expect("Failed to create app directory on local filesystem for mounting as a volume.");

        let config_path = format!("{}/config", app_path);
        write_file(&config_path, config).await;

        let catalog_path = format!("{}/catalog", app_path);
        write_file(&catalog_path, catalog).await;

        let mut container = Container::new(docker);
        container.image_name(Self::IMAGE);

        let command = vec![
            "read",
            "--config",
            "/app/config",
            "--catalog",
            "/app/catalog",
        ];

        // Create the paths to the `app` folder that will be used for binding the folder to the
        // container as a volume.
        let volume_path = format!("{}:/app/", app_path);

        // Create container, run the command and receive a stream that will be used to read from
        // the stdout of the container.
        let read = container
            .start_container(command, Some(vec![&volume_path]))
            .await;

        // NOTE: We are not cleaning the config and catalog files from our `app/` folder because
        // the container still lives on after we exit this method. At this point, I'm not sure if
        // the container volume copies the file into the container, or just links them to the
        // container.
        // remove_file(&config_path).await;
        // remove_file(&catalog_path).await;

        // Convert the type of the `read` stream from `Bytes` to `String`.
        read.map(|s| String::from_utf8(s.unwrap().to_vec()).unwrap())
            .boxed()
    }
}

async fn create_app_folder(path: &str) -> Result<(), std::io::Error> {
    // Check if the `app` folder exists. We will bind this folder to the container as a volume.
    if !std::path::Path::new(path).exists() {
        create_dir(path)
            .await
            .expect("Failed to create new folder.")
    }
    Ok(())
}

async fn write_file(file_path: &str, content: &Value) {
    // Create the config file.
    let mut file = File::create(file_path)
        .await
        .expect("Failed to create config file.");

    // Write the config JSON to the file.
    let content: String = serde_json::to_string(content).unwrap();
    file.write_all(content.as_bytes())
        .await
        .expect("Failed to write JSON config to file.");
}

async fn remove_file(path: &str) {
    if Path::new(path).exists() {
        fs::remove_file(path)
            .await
            .expect("Failed to remove file from disk.");
    }
}
