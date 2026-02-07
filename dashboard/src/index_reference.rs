use futures::{FutureExt, future::join_all};
use gloo_net::http::Request;
use serde_json::Value;
use serde_wasm_bindgen::to_value;
use shared::statistics::EvaluatedStatistics;
use std::collections::HashSet;
use web_sys::console;
use yew::Properties;

use crate::models::plot_settings::{PlotSettings, Sub, SubSettings};

// TODO: maybe handle file loading errors better
async fn load_files(url: Vec<String>) -> Vec<String> {
    let futures = url.into_iter().map(|url| async move {
        let text = match Request::get(&url).send().await {
            Ok(response) => response.text().await.expect("Could not get response text"),
            Err(error) => {
                console::error_1(
                    &to_value(&format!("Failed to fetch file {}: Error {:?}", url, error))
                        .expect("Could not send error message to console"),
                );
                "".to_string()
            }
        };
        println!("loaded one");
        text
    });
    join_all(futures).await
}

#[derive(Properties, Clone, PartialEq, Debug)]
struct ResultFile {
    run: String,
    filename: String,
}

impl ResultFile {
    fn from(full_path: String) -> Self {
        let mut parts = full_path.split("/");
        let results_path = parts
            .next()
            .expect("Expecting results to be the same every time");
        assert_eq!("results", results_path, "Results path should be results");
        let run = parts
            .next()
            .expect("Could not get run from results path")
            .into();
        let filename = parts
            .next()
            .expect("Could not get filename from results path")
            .into();
        assert!(parts.next().is_none(), "Too many parts in results path");
        Self { run, filename }
    }

    fn run(&self) -> String {
        self.run.clone()
    }

    fn path(&self) -> String {
        format!("/results/{}/{}", self.run, self.filename)
    }
}

#[derive(Properties, Clone, PartialEq, Debug)]
pub struct ResultFiles {
    result_files: Vec<ResultFile>,
}

impl ResultFiles {
    pub fn from(full_paths: Vec<String>) -> Self {
        let result_files = full_paths.into_iter().map(ResultFile::from).collect();
        Self { result_files }
    }

    pub fn len(&self) -> usize {
        self.result_files.len()
    }

    pub async fn load(&self) -> Vec<EvaluatedStatistics> {
        let all_paths = self.result_files.iter().map(|r| r.path()).collect();
        let files = load_files(all_paths).await;
        let evaluated_statistics = files
            .iter()
            .map(|f| serde_json::from_str(f).expect("Could not deserialize file"))
            .collect();
        evaluated_statistics
    }

    pub async fn load_strings(&self) -> Vec<String> {
        let all_paths = self.result_files.iter().map(|r| r.path()).collect();
        let files = load_files(all_paths).await;
        files
    }

    pub fn runs(&self) -> Vec<String> {
        let unique_runs: HashSet<_> = self.result_files.iter().map(|r| r.run()).collect();
        unique_runs.into_iter().collect()
    }

    pub fn only_latest_run(&self) -> Self {
        let runs = self.runs();
        let latest_run = runs.into_iter().max().expect("No latest run found");
        self.filter_by_run(&latest_run)
    }

    pub fn filter_by_run(&self, run: &String) -> Self {
        let filtered_results = self
            .result_files
            .iter()
            .filter(|r| r.run() == run.as_str())
            .cloned()
            .collect();
        Self {
            result_files: filtered_results,
        }
    }

    pub async fn load_by_settings(&self, settings: PlotSettings) -> Vec<EvaluatedStatistics> {
        let all = self.load_strings().await;
        let all_raw = all
            .iter()
            .filter(|s| complies_with(s, &settings))
            .map(|s| EvaluatedStatistics::from_string(s))
            .collect();
        all_raw
    }
}

fn complies_with(statistics_raw: &str, plot_settings: &PlotSettings) -> bool {
    let statistics: Value = match serde_json::from_str(statistics_raw) {
        Ok(v) => v,
        Err(_) => return false,
    };
    matches_sub(
        &statistics,
        &Sub::SubSettings(SubSettings {
            subsettings: plot_settings.settings_map.clone(),
        }),
    )
}

fn matches_sub(value: &Value, sub: &Sub) -> bool {
    match (value, sub) {
        (Value::Object(map), Sub::SubSettings(settings)) => settings
            .subsettings
            .iter()
            .all(|(k, s)| map.get(k).map_or(false, |v| matches_sub(v, s))),
        (_, Sub::Variations(variations)) => {
            let key = shallow_json_to_key(value);
            variations.variations.get(&key) == Some(&true)
        }
        _ => false,
    }
}

fn shallow_json_to_key(value: &Value) -> String {
    match value {
        Value::Null => panic!("No null should be in json file"),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.to_string(),
        _ => panic!("Expected shallow json"),
    }
}
