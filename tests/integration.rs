use dirtcrunch::*;
use serde_json::Value;

// This test is basically an example of how to use this crate.

struct File {}

impl File {
    pub fn new() -> Self {
        Self {}
    }
}

impl Source for File {
    const CONNECTOR: &'static str = "airbyte/source-file";
}

#[tokio::test]
async fn test() {
    let file = File::new();

    let result = file.specs().await;

    let result: Value = serde_json::from_str(result.as_str()).unwrap();
    let result = result
        .get("spec")
        .unwrap()
        .get("connectionSpecification")
        .unwrap()
        .get("required")
        .unwrap();

    assert_eq!(result.as_array().unwrap().len(), 4);
}
