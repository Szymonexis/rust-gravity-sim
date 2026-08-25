use schemars::generate::SchemaSettings;
use serde::Serialize;
use serde_json::{Value, json};

use crate::config::AppConfig;

pub fn schema() -> String {
    let mut root = SchemaSettings::draft07()
        .into_generator()
        .root_schema_for::<AppConfig>()
        .to_value();

    if let Some(root) = root.as_object_mut() {
        root.remove("required");

        if let Some(properties) = root.get_mut("properties").and_then(Value::as_object_mut) {
            properties.insert(
                "$schema".to_owned(),
                json!({
                    "type": "string",
                    "description": "Path to this schema. Ignored by the loader."
                }),
            );
        }
    }

    tidy(&mut root);

    let mut out = Vec::new();
    let mut serializer = serde_json::Serializer::with_formatter(
        &mut out,
        serde_json::ser::PrettyFormatter::with_indent(b"\t"),
    );
    root.serialize(&mut serializer)
        .expect("a schema is always serialisable");
    out.push(b'\n');

    String::from_utf8(out).expect("serde_json writes utf-8")
}

fn tidy(value: &mut Value) {
    match value {
        Value::Object(obj) => {
            if let Some(Value::String(description)) = obj.get_mut("description") {
                *description = description
                    .split("\n\n")
                    .map(|paragraph| paragraph.split('\n').collect::<Vec<_>>().join(" "))
                    .collect::<Vec<_>>()
                    .join("\n\n");
            }

            for branch in obj
                .get_mut("oneOf")
                .and_then(Value::as_array_mut)
                .map(Vec::as_mut_slice)
                .unwrap_or_default()
                .iter_mut()
                .filter_map(Value::as_object_mut)
            {
                if let Some([Value::String(variant)]) = branch
                    .get("required")
                    .and_then(Value::as_array)
                    .map(Vec::as_slice)
                {
                    let title = variant.clone();
                    branch.entry("title").or_insert(Value::String(title));
                }
            }

            for nested in obj.values_mut() {
                tidy(nested);
            }
        }

        Value::Array(items) => items.iter_mut().for_each(tidy),

        _ => {}
    }
}
