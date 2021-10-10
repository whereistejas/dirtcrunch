use crate::container::Container;
use crate::util::source::create_objects;
use futures::{future, TryStreamExt};
use serde::Deserialize;
use serde_json::Value;
use serde_yaml;
use shiplift::Docker;

/// This method returns the SPECS of the source connector.
async fn get_specs(connector: &str) -> Value {
    let docker = Docker::new();
    let mut container = Container::new(&docker);

    // Set the image name.
    container.imagename(connector);
    container.prepare_image().await;

    let read = container.start_container(vec!["spec"], None).await;

    let result_bytes = read
        .try_collect::<Vec<_>>()
        .await
        .expect("Failed to read command output from docker container.");

    let result = result_bytes
        .iter()
        .map(|s| String::from_utf8(s.to_vec()).unwrap())
        .collect::<Vec<_>>();

    // Search for the SPECS JSON object.
    let search_string = "{\"type\": \"SPEC\",";
    let spec = result
        .iter()
        .find(|s| s.contains(search_string))
        .expect("Could not find specs.");
    let start_index = spec.find(search_string).expect("Could not find specs.");

    let spec = spec.split_at(start_index).1;

    container
        .delete_container(false)
        .await
        .expect("Failed to remove connector.");

    serde_json::from_str(spec).expect("Failed to parse the SPECS JSON object.")
}

/// This struct stores the information from the following file in the main airbyte repository:
/// `airbyte/airbyte-config/init/src/main/resources/seed/source_definitions.yaml`
#[derive(Deserialize, Debug)]
struct Source {
    name: String,
    #[serde(rename(deserialize = "sourceDefinitionId"))]
    source_definition_id: String,
    #[serde(rename(deserialize = "dockerRepository"))]
    docker_repository: String,
    #[serde(rename(deserialize = "dockerImageTag"))]
    docker_image_tag: String,
    #[serde(rename(deserialize = "sourceType"))]
    source_type: String,
    #[serde(rename(deserialize = "documentationUrl"))]
    documentation_url: String,
}

/// This method accepts a list of source connectors and returns a string of all the structs and
/// trait implementations for the connectors in the given list.
pub async fn get_objects(source_list: serde_yaml::Value) -> String {
    let mut sources: Vec<Source> = serde_yaml::from_value(source_list).unwrap();

    // NOTE: For now, we will build only the first 3 sources. This is only for testing purposes.
    sources.drain(3..);

    // Collect all `spec` commands for the given connectors into one vector.
    let tasks = sources
        .iter()
        .map(|source| get_specs(&source.docker_repository))
        .collect::<Vec<_>>();

    // Run all tasks, parallellllllll-ly.
    let specs = future::join_all(tasks).await;

    // Convert SPECS JSON objects into structs and trait impls source code.
    specs
        .iter()
        .zip(sources.iter())
        .map(|(spec, source)| {
            let iter = source.name.split_whitespace();

            let mut name = String::new();
            for value in iter {
                name.push_str(value);
            }

            create_objects(name.as_str(), &source.docker_repository, spec.clone())
        })
        .collect::<Vec<_>>()
        .join("\n")
}
