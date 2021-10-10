use crate::container::Container;
use async_trait::async_trait;
use futures::{Stream, StreamExt, TryStreamExt};
use serde_json::Value;
use shiplift::Docker;
use std::{env, path::Path, pin::Pin};
use tokio::{
    fs::{self, create_dir, File},
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
    async fn discover(&self, config: &Value) -> Vec<String> {
        // Set path for the `app` folder.
        let app_path = format!("{}/app", env::current_dir().unwrap().to_str().unwrap());

        create_app_folder(&app_path)
            .await
            .expect("failed to create app directory on local filesystem for mounting as a volume.");

        let config_path = format!("{}/config", app_path);
        write_file(&config_path, config).await;

        let docker = Docker::new();
        let mut container = Container::new(&docker);
        container.imagename(Self::IMAGE);

        // Create the paths to the `app` folder that will be used for binding the folder to the
        // container as a volume.
        let volume_path = format!("{}:/app/", app_path);

        // Create container, run the command and receive a stream that will be used to read from
        // the stdout of the container.
        let read = container
            .start_container(
                vec!["discover", "--config", "/app/config"],
                Some(vec![&volume_path]),
            )
            .await;

        // Read command output as a byte vector.
        let result_bytes = read
            .try_collect::<Vec<_>>()
            .await
            .expect("Failed to read command output from docker container.");

        // Remove config file.
        remove_file(&config_path).await;

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
        container.imagename(Self::IMAGE);

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

        // Convert the type of the `read` stream from `&[u8]` to `String`.
        read.map(|s| String::from_utf8(s.unwrap().to_vec()).unwrap())
            .boxed()
    }
}

async fn create_app_folder(path: &str) -> Result<(), std::io::Error> {
    // Check if the `app` folder exists. We will bind this folder to the container as a volume.
    if !std::path::Path::new(path).exists() {
        create_dir(path).await
    } else {
        Ok(())
    }
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
