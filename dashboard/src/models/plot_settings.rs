use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;
use serde_json::{Value, from_str};
use yew::Properties;

use crate::utils::console::console_error;

#[derive(Properties, Debug, PartialEq, Clone)]
pub struct PlotSettings {
    pub settings_map: IndexMap<Vec<String>, bool>,
}

impl PlotSettings {
    pub fn from_result_files_content(files_content: Vec<String>) -> Self {
        let mut paths = HashSet::new();
        for file_content in &files_content {
            let mut value: Value = from_str(file_content).expect("Could not parse json file");
            // remove the fitness traces
            value
                .as_object_mut()
                .expect("Cannot modify non json object")
                .remove("evaluated_traces");
            let file_paths = collect_paths_from_json(&value);
            paths.extend(file_paths);
        }
        let mut settings_map = IndexMap::new();
        for path in paths {
            settings_map.insert(path, false);
        }
        if files_content.len() > 1 {
            console_error(&format!("{:#?}", &settings_map));
        }
        Self { settings_map }
    }

    pub fn paths(&self) -> Vec<Vec<String>> {
        self.settings_map
            .iter()
            .map(|(path, _)| path)
            .cloned()
            .collect()
    }

    pub fn from_result_file_content(file_content: String) -> Self {
        Self::from_result_files_content(vec![file_content])
    }

    pub fn complies_with(&self, plot_settings: &PlotSettings) -> bool {
        for setting in self.settings_map.keys() {
            let plot_setting = plot_settings.settings_map.get(setting);
            match plot_setting {
                None | Some(false) => return false,
                Some(true) => {}
            };
        }
        true
    }

    pub fn toggle(&mut self, path: Vec<String>) {
        let setting = self
            .settings_map
            .get_mut(&path)
            .expect("Setting does not exist in plot settings");
        *setting = !*setting;
    }
}

fn collect_paths_from_json(value: &Value) -> Vec<Vec<String>> {
    let mut paths = Vec::new();
    let mut path = Vec::new();
    subpath(value, &mut path, &mut paths);
    paths
}

fn subpath(value: &Value, path: &mut Vec<String>, result: &mut Vec<Vec<String>>) {
    match value {
        Value::Object(m) => {
            for (key, value) in m {
                path.push(key.clone());
                subpath(value, path, result);
                path.pop();
            }
        }
        Value::Array(a) => {
            for value in a {
                subpath(value, path, result);
            }
        }
        Value::String(s) => {
            let mut row = path.clone();
            row.push(s.clone());
            result.push(row);
        }
        Value::Number(n) => {
            let mut row = path.clone();
            row.push(n.to_string());
            result.push(row);
        }
        Value::Bool(b) => {
            let mut row = path.clone();
            row.push(b.to_string());
            result.push(row);
        }
        Value::Null => {
            panic!("There should be no null values in loaded json");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{read_dir, read_to_string};

    #[test]
    fn test_collect_paths() {
        let s = "{\"settings\": {\"third\": {\"a\": 100, \"b\": [\"200\", \"300\"]}}}".to_string();
        let value = serde_json::from_str(&s).expect("Cannot turn string into json value");
        dbg!(&value);
        let paths = collect_paths_from_json(&value);
        assert_eq!(
            paths,
            vec![
                vec!["settings", "third", "a", "100"],
                vec!["settings", "third", "b", "200"],
                vec!["settings", "third", "b", "300"],
            ]
        );
    }

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
    }

    #[test]
    fn test_merge_real_settings() {}
}
