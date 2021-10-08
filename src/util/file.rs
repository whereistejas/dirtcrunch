use serde_json::Value;

#[derive(Debug)]
struct Field<'a> {
    // Field name.
    name: &'a str,
    // Field type.
    ftype: &'a str,
}

fn create_fields(json: &Value) -> Vec<Field> {
    let required = json
        .pointer("/spec/connectionSpecification/required")
        .unwrap()
        .as_array()
        .unwrap();

    let required = required
        .iter()
        .map(|s| s.as_str().unwrap())
        .collect::<Vec<_>>();

    let fields = required
        .iter()
        .map(|field| {
            let pointer = format!("/spec/connectionSpecification/properties/{}/type", field);
            let ftype = json.pointer(&pointer).unwrap().as_str().unwrap();

            Field { name: field, ftype }
        })
        .collect::<Vec<Field>>();

    fields
}

// Generate the file which we need to add
pub fn create_file(content: String) -> String {
    r#"use dirtcrunch::{AirbyteReturn, Source};

CONTENT
"#
    .replace("CONTENT", &content)
}

pub fn create_objects(name: &str, image: &str, json: Value) -> String {
    let fields = create_fields(&json);
    let config = create_config(name, &fields);

    r#"CONFIG
pub struct NAME {}

impl NAME {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl<'a> Source<'a, ConfigNAME> for NAME {
    const IMAGE: &'a str = "IMAGE_NAME";
    fn specs(&self) -> AirbyteReturn { 
        let value = serde_json::Value::String(START SPECS END.to_string());

        AirbyteReturn::Config(value)
    }
    fn check(&self, config: &ConfigNAME) -> AirbyteReturn { todo!() }
    fn read(&self, config: &ConfigNAME) -> AirbyteReturn { todo!() }
}
"#
    .replace("NAME", name)
    .replace("IMAGE_NAME", image)
    .replace("CONFIG", &config)
    .replace("START", "r#\"")
    .replace("END", "\"#")
    .replace("SPECS", &json.to_string())
}

fn create_config(name: &str, fields: &[Field]) -> String {
    r#"#[derive(serde::Serialize, serde::Deserialize)]
pub struct ConfigNAME {
FIELD
}
"#
    .replace("NAME", name)
    .replace(
        "FIELD",
        {
            let mut line = String::new();

            for field in fields {
                let ftype = match field.ftype {
                    "integer" => "i32".to_string(),
                    "object" | "string" => "String".to_string(),
                    _ => panic!("Encountered unknown type."),
                };
                line.push_str(format!("\tpub {}: {},\n", field.name, ftype).as_str());
            }

            line
        }
        .as_ref(),
    )
}
