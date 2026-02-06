use std::collections::HashMap;

use indexmap::IndexMap;
use serde_json::{Value, from_str};
use yew::Properties;

use crate::utils::console::console_error;

#[derive(Debug, PartialEq)]
pub enum Converted {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Converted>),
    Map(HashMap<String, Converted>),
}

fn convert_json_value(value: Value) -> Converted {
    match value {
        Value::Null => Converted::Null,
        Value::Bool(b) => Converted::Bool(b),
        Value::Number(n) => Converted::Number(n.as_f64().expect("Cannot read json number")),
        Value::String(s) => Converted::String(s),
        Value::Array(a) => Converted::Array(a.into_iter().map(convert_json_value).collect()),
        Value::Object(o) => Converted::Map(
            o.into_iter()
                .map(|(k, v)| (k, convert_json_value(v)))
                .collect(),
        ),
    }
}

fn merge_converted(first: Converted, second: Converted) -> Converted {
    match (first, second) {
        (Converted::Null, Converted::Null) => panic!("There should be no null value in json"),
        (Converted::Bool(a), Converted::Bool(b)) if a == b => Converted::Bool(a),
        (Converted::Number(a), Converted::Number(b)) if a == b => Converted::Number(a),
        (Converted::String(a), Converted::String(b)) if a == b => Converted::String(a),
        (Converted::Array(a), Converted::Array(b)) if a.len() == b.len() => Converted::Array(
            a.into_iter()
                .zip(b)
                .map(|(f, s)| merge_converted(f, s))
                .collect(),
        ),
        (Converted::Array(mut a), b) => {
            push_dedup(&mut a, b);
            Converted::Array(a)
        }
        (a, Converted::Array(mut b)) => {
            insert_dedup(&mut b, 0, a);
            Converted::Array(b)
        }
        (Converted::Map(mut a), Converted::Map(b)) => {
            for (key, value_b) in b {
                match a.remove(&key) {
                    Some(value_a) => a.insert(key, merge_converted(value_a, value_b)),
                    None => a.insert(key, value_b),
                };
            }
            Converted::Map(a)
        }
        (a, b) => Converted::Array(vec![a, b]),
    }
}

fn insert_dedup(vec: &mut Vec<Converted>, position: usize, value: Converted) {
    if !vec.contains(&value) {
        vec.insert(position, value);
    }
}

fn push_dedup(vec: &mut Vec<Converted>, value: Converted) {
    if !vec.contains(&value) {
        vec.push(value);
    }
}

fn merge_converted_multiple(mut values: impl Iterator<Item = Converted>) -> Converted {
    let first = values.next().expect("Cannot merge empty iterator");
    values.fold(first, merge_converted)
}

impl PlotSettings {
    pub fn from_result_files_content(files_content: Vec<String>) -> Self {
        let mut all_converted = Vec::new();
        for file_content in files_content {
            let mut value: Value = from_str(&file_content).expect("Could not parse json file");
            // remove the fitness entries
            value
                .as_object_mut()
                .expect("Cannot modify non json object")
                .remove("evaluated_traces");
            let converted = convert_json_value(value);
            all_converted.push(converted);
        }
        let merged = merge_converted_multiple(all_converted.into_iter());
        let plot_settings = Self::from_converted(merged);
        plot_settings
    }

    pub fn from_converted(converted: Converted) -> Self {
        if let Converted::Map(map) = converted {
            PlotSettings {
                settings_map: convert_map(map),
            }
        } else {
            panic!("Top level converted must be a map to fit plot settings");
        }
    }
}

fn convert_map(map: HashMap<String, Converted>) -> IndexMap<String, Sub> {
    let mut index_map = IndexMap::new();
    for (key, value) in map {
        let sub = match value {
            Converted::Null => panic!("Null value is not allowed in plot settings"),
            Converted::Bool(b) => {
                let mut variations = IndexMap::new();
                variations.insert(key.clone(), b);
                Sub::Variations(Variations { variations })
            }
            Converted::Number(_) | Converted::String(_) => {
                let mut variations = IndexMap::new();
                let key_string = match value {
                    Converted::Number(n) => n.to_string(),
                    Converted::String(s) => s,
                    _ => panic!("Only number and string are reachable"),
                };
                variations.insert(key_string, true);
                Sub::Variations(Variations { variations })
            }
            Converted::Array(a) => {
                let mut variations = IndexMap::new();
                for item in a {
                    match item {
                        Converted::Number(n) => variations.insert(n.to_string(), true),
                        Converted::String(s) => variations.insert(s, true),
                        _ => panic!("Only number and string are reachable"),
                    };
                }
                variations.sort_by(|key1, _, key2, _| {
                    match (key1.parse::<usize>(), key2.parse::<usize>()) {
                        (Ok(number1), Ok(number2)) => number1.cmp(&number2),
                        _ => key1.cmp(key2),
                    }
                });
                Sub::Variations(Variations { variations })
            }
            Converted::Map(m) => {
                let mut subsettings = convert_map(m);
                subsettings.sort_by(|key1, _, key2, _| key1.cmp(key2));
                Sub::SubSettings(SubSettings { subsettings })
            }
        };
        index_map.insert(key, sub);
    }
    index_map
}

#[derive(Properties, Debug, PartialEq, Clone)]
pub struct PlotSettings {
    pub settings_map: IndexMap<String, Sub>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Sub {
    SubSettings(SubSettings),
    Variations(Variations),
}

#[derive(Properties, Debug, PartialEq, Clone)]
pub struct SubSettings {
    pub subsettings: IndexMap<String, Sub>,
}

#[derive(Properties, Debug, PartialEq, Clone)]
pub struct Variations {
    pub variations: IndexMap<String, bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{read_dir, read_to_string};

    #[test]
    fn test_merge_simple() {
        let files = vec![
            "{\"settings\": {\"pop\": \"10\"}}".to_string(),
            "{\"settings\": {\"pop\": \"100\"}}".to_string(),
            "{\"settings\": {\"pop\": \"1000\"}}".to_string(),
            "{\"settings\": {\"gen\": \"10\"}}".to_string(),
            "{\"settings\": {\"gen\": \"100\"}}".to_string(),
            "{\"settings\": {\"third\": {\"a\": \"10\", \"b\":\"20\"}}}".to_string(),
            "{\"settings\": {\"third\": {\"a\": \"100\", \"b\":\"200\"}}}".to_string(),
        ];
        PlotSettings::from_result_files_content(files);
    }

    #[test]
    fn test_merge_real_settings() {
        let files: Vec<String> = read_dir("results/.20260131144346/")
            .expect("No files at given path")
            .map(|e| {
                let path = e.unwrap().path();
                read_to_string(&path).unwrap()
            })
            .collect();
        let plot_settings = PlotSettings::from_result_files_content(files);
        dbg!(&plot_settings);
        todo!()
    }
}
