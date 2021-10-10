/// This structure stores the varies fields for the `Config` struct.
#[derive(Debug)]
struct Field<'a> {
    // Field name.
    name: &'a str,
    // Field type.
    ftype: &'a str,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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
