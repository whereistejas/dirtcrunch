use serde_json::Value;

/// Put the Source trait objects in a `.rs` file.
pub fn create_file(content: String) -> String {
    r#"use dirtcrunch::Source;

CONTENT
"#
    .replace("CONTENT", &content)
}

/// Create the connector struct and implement the Source trait for that struct.
pub fn create_objects(name: &str, image: &str, json: Value) -> String {
    r#"pub struct NAME {}

impl NAME {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl<'a> Source<'a> for NAME {
    const IMAGE: &'a str = "IMAGE_NAME";
    fn specs(&self) -> serde_json::Value { 
        serde_json::Value::String(START SPECS END.to_string())
    }
}
"#
    .replace("IMAGE_NAME", image)
    .replace("NAME", name)
    .replace("START", "r#\"")
    .replace("END", "\"#")
    .replace("SPECS", &json.to_string())
}
