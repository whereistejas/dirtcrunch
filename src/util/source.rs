use crate::util::specs::get_objects;
use serde_json::Value;

/// Put the Source trait objects in a `.rs` file.
pub async fn create_file(source_list: serde_yaml::Value) -> String {
    let content: String = get_objects(source_list).await;

    format!(
        r#"
use dirtcrunch::Source;
{CONTENT}

"#,
        CONTENT = content
    )
}

/// Create the connector struct and implement the Source trait for that struct.
pub(crate) fn create_objects(name: &str, image: &str, json: Value) -> String {
    format!(
        r#"
pub struct {NAME};

impl {NAME} {{
    pub fn new() -> Self {{
        Self
    }}
}}

#[async_trait::async_trait]
impl Source for {NAME} {{
    const IMAGE: &'static str = "{IMAGE_NAME}";
    fn specs(&self) -> serde_json::Value {{
        serde_json::json!({START} {SPECS} {END}.to_string())
    }}
}}
"#,
        IMAGE_NAME = image,
        NAME = name,
        START = "r#\"",
        END = "\"#",
        SPECS = &json.to_string(),
    )
}
