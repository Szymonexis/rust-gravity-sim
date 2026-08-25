use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

pub trait FromJsonObject: Default {
    fn from_json_object(obj: &Map<String, Value>, path: &str) -> Self;
}

pub fn leaf<T: DeserializeOwned>(
    obj: &Map<String, Value>,
    key: &str,
    path: &str,
    fallback: T,
) -> T {
    let Some(value) = obj.get(key) else {
        return fallback;
    };

    match T::deserialize(value) {
        Ok(parsed) => parsed,
        Err(err) => {
            eprintln!(
                "config: `{}` is invalid ({err}); using default",
                join(path, key)
            );
            fallback
        }
    }
}

pub fn section<T: FromJsonObject>(obj: &Map<String, Value>, key: &str, path: &str) -> T {
    match obj.get(key) {
        None => T::default(),
        Some(Value::Object(inner)) => T::from_json_object(inner, &join(path, key)),
        Some(other) => {
            eprintln!(
                "config: `{}` should be an object but is {}; using defaults",
                join(path, key),
                kind_of(other)
            );
            T::default()
        }
    }
}

pub fn warn_unknown_keys(obj: &Map<String, Value>, known: &[&str], path: &str) {
    for key in obj.keys().filter(|key| !known.contains(&key.as_str())) {
        eprintln!("config: ignoring unknown key `{}`", join(path, key));
    }
}

pub fn join(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_owned()
    } else {
        format!("{path}.{key}")
    }
}

fn kind_of(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}
