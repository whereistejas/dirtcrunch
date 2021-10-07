use crate::container::Container;
use futures::TryStreamExt;
use serde_json::Value;

pub async fn get_specs(connector: &str) -> Value {
    let mut container = Container::new();
    container.prepare_image(connector).await;

    let read = container.start_container("spec").await;

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
        .delete_container()
        .await
        .expect("Failed to remove connector.");

    serde_json::from_str(spec.1).unwrap()
}

#[derive(Debug)]
struct Field<'a> {
    name: &'a str,
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

pub fn create_file(name: &str, json: Value) -> String {
    let fields = create_fields(&json);
    let config = create_config(&fields);

    r#"use dirtcrunch::{Command, Source};
use serde_json::Value;
use async_trait::async_trait;

CONFIG

struct NAME {}

impl NAME {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Source<Config> for NAME {
    fn specs(&self) -> Command
    { 
        let value: Value = Value::String(START SPECS END.to_string());

        Command::Spec(value)
    }
    async fn discover(&self, config: &Config) -> Command { todo!() }
    fn read(&self, config: &Config) -> Command { todo!() }
}
"#
    .replace("NAME", name)
    .replace("CONFIG", &config)
    .replace("START", "r#\"")
    .replace("END", "\"#")
    .replace("SPECS", &json.to_string())
}

fn create_config(fields: &[Field]) -> String {
    let structure = r#"pub struct Config {
FIELD
}
"#
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
                line.push_str(format!("{}:{},\n", field.name, ftype).as_str());
            }

            line
        }
        .as_ref(),
    );

    structure
}
