use std::fs;

use crate::{util, version::LiteLoaderVersions};

const LITELOADER_VERSIONS_JSON: &'static str = "https://dl.liteloader.com/versions/versions.json";

pub async fn handle(opt_version: Option<String>, opt_loader_version: Option<String>, limit: String) {
    let versions_json_text = util::download_text_no_save_async(LITELOADER_VERSIONS_JSON, "Downloaded liteloader versions json".to_owned()).await.expect("Failed to download liteloader versions json");
    tokio::fs::write("ver.json", versions_json_text.to_string()).await.unwrap();
    let versions_json: LiteLoaderVersions = serde_json::from_str(&versions_json_text).expect("Failed to parse liteloader versions");
    let versions = versions_json.versions;
    let mut version = String::new();
    if opt_version.is_some() {
        if !versions.iter().any(|i| *i.0 == opt_version.clone().unwrap()) {
            eprintln!("The version you provided doesn't exist");
            std::process::exit(-1);
        }
        version = opt_version.clone().unwrap();
    } else {
        let mut max = 0;
        for version in &versions {
            if version.0.split('.').collect::<Vec<&str>>()[1].parse::<i32>().unwrap() > max {
                max = version.0.split('.').collect::<Vec<&str>>()[1].parse::<i32>().unwrap();
            }
        }
        version = format!("1.{}", max);
    }
    println!("{}", version);
    if versions[&version].artefacts.is_some() {
    }
}