use std::{fs, path::PathBuf};

use crate::statistics::Statistic;

pub fn analyze(foldername: String) {
    let entries = fs::read_dir(&foldername).expect("No folder found").filter_map(Result::ok);
    let filepaths: Vec<PathBuf> = entries.map(|e| e.path()).filter(|p| p.is_file()).collect();
    let statistics: Vec<Statistic> = filepaths.into_iter().map(|f| deserialize_file(f)).collect();
    let maximum = statistics.iter().map(|s| s.maximum()).reduce(|a, b| if a > b { a } else { b }).expect("No maximum");
    let average = statistics.iter().map(|s| s.average()).sum::<f64>() / statistics.len() as f64;
    dbg!(foldername);
    dbg!(maximum);
    dbg!(average);
}

fn deserialize_file(filepath: PathBuf) -> Statistic {
    let text = fs::read_to_string(filepath).expect("Could not read file");
    let statistic = serde_json::from_str(&text).expect("Could not deserialize statistic");
    statistic
}