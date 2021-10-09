use crate::container::Container;
use futures::TryStreamExt;
use serde_json::Value;
use shiplift::Docker;

/// This method is meant to be used in `build.rs` to generate `source.rs`.
pub async fn get_specs(connector: &str) -> Value {
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

    let search_string = "{\"type\": \"SPEC\",";
    let spec = result
        .iter()
        .find(|s| s.contains(search_string))
        .expect("Could not find specs.");
    let start_index = spec.find(search_string).expect("Could not find specs.");

    let spec = spec.split_at(start_index);

    container
        .delete_container(false)
        .await
        .expect("Failed to remove connector.");

    serde_json::from_str(spec.1).expect("Failed to parse the SPECS JSON object.")
}
