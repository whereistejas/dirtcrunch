use crate::container::Container;
use crate::util::source::create_objects;
use futures::future;
use log::info;
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;
use serde_yaml;
use shiplift::Docker;
use tokio::io::AsyncBufReadExt;
use tokio_util::io::StreamReader;

/// This method returns the SPECS of the source connector.
async fn get_specs(connector: &str, tag: &str) -> Result<Value, String> {
    let docker = Docker::new();
    let mut container = Container::new(&docker);

    info!("Preparing connector: {}", connector);

    // Set the image name and tag and pull the image from docker-hub.
    container.image_name(connector);
    container.image_tag(tag);
    container.prepare_image().await;

    info!("Finished preparing connector: {}", connector);

    info!("Start searching for SPECS object: {}.", connector);

    let stream = container.start_container(vec!["spec"], None).await;

    let mut reader = StreamReader::new(stream);

    let mut line = String::new();
    while let Ok(result) = reader.read_line(&mut line).await {
        if result != 0 {
            let regex = Regex::new(r#"\{"type"\s*:\s*"SPEC"\s*,"#)
                .expect("Unable to compile given regular expression.");

            if regex.is_match(line.as_str()) {
                break;
            } else {
                line.clear();
            }
        } else {
            panic!("Could not find SPEC object.")
        }
    }

    info!("Finish searching for SPECS object: {}.", connector);

    Ok(serde_json::from_str(&line).expect("Failed to parse the SPECS JSON object."))
}

/// This struct stores the information from the following file in the main airbyte repository:
/// `airbyte/airbyte-config/init/src/main/resources/seed/source_definitions.yaml`
#[derive(Deserialize, Debug)]
struct Source {
    name: String,
    #[allow(dead_code)]
    #[serde(rename(deserialize = "sourceDefinitionId"))]
    source_definition_id: String,
    #[serde(rename(deserialize = "dockerRepository"))]
    docker_repository: String,
    #[serde(rename(deserialize = "dockerImageTag"))]
    docker_image_tag: String,
    #[allow(dead_code)]
    #[serde(rename(deserialize = "sourceType"))]
    source_type: String,
    #[allow(dead_code)]
    #[serde(rename(deserialize = "documentationUrl"))]
    documentation_url: String,
}

/// This method accepts a list of source connectors and returns a string of all the structs and
/// trait implementations for the connectors in the given list.
pub(super) async fn get_objects(source_list: serde_yaml::Value) -> String {
    let mut sources: Vec<Source> = serde_yaml::from_value(source_list).unwrap();

    // NOTE: For now, we will build only the first 5 sources. This is only for testing purposes.
    // TODO: Remove this when we publish this crate for good.
    if sources.len() > 5 {
        sources.drain(5..);
    }

    // Collect all `spec` commands for the given connectors into one vector.
    let tasks = sources
        .iter()
        .map(|source| get_specs(&source.docker_repository, &source.docker_image_tag))
        .collect::<Vec<_>>();

    info!("Start running all tasks.");

    // Run all tasks, parallellllllll-ly.
    let specs = future::join_all(tasks).await;

    info!("Finish running all tasks.");

    // Convert SPECS JSON objects into structs and trait impls source code.
    specs
        .iter()
        .zip(sources.iter())
        .map(|(spec, source)| {
            let mut words = source.name.split_whitespace().collect::<Vec<_>>();

            // Use only the first 3 words from the source's name to create the struct's name.
            if words.len() > 3 {
                words.drain(3..);
            }

            info!("Start preparing objects: {}", &source.docker_repository);
            let object = create_objects(
                &words.join(""),
                format!("{}:{}", &source.docker_repository, &source.docker_image_tag).as_str(),
                spec.as_ref().unwrap().clone(),
            );
            info!("Finish preparing objects: {}", &source.docker_repository);

            object
        })
        .collect::<Vec<_>>()
        .join("\n")
}
