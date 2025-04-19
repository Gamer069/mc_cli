use std::{fs::{self, File}, path::{Path, PathBuf}};

use directories::ProjectDirs;
use serde::Deserialize;

use crate::mem;

const VANILLA_MANIFEST: &'static str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

#[derive(Deserialize, Debug)]
pub struct VanillaVersion {
    pub id: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct VanillaLatest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Deserialize, Debug)]
pub struct VanillaManifest {
    pub latest: VanillaLatest,
    pub versions: Vec<VanillaVersion>,
}

pub fn get_ver_json_url(manifest: VanillaManifest, version: String) -> String {
    let mut found = false;
    let mut url = "".to_owned();
    manifest.versions.iter().any(|item| {
        found = item.id.trim() == version.trim();
        if found {
            url = item.url.clone();
        }
        found
    });
    if !found {
        eprintln!("FATAL: The specified version does not exist.");
        std::process::exit(-1);
    };
    url
}

pub fn get_manifest() -> VanillaManifest {
    let req = reqwest::blocking::get(VANILLA_MANIFEST).expect("Failed to get vanilla version manifest");
    let body = req.text().unwrap();
    serde_json::from_str(&body).unwrap()
}

pub fn launch(version_dir: PathBuf, limit: String) {
}

pub fn create_dirs(vers: PathBuf, ver: PathBuf) {
    let _ = fs::create_dir_all(vers.clone());
    let _ = fs::create_dir(ver);
}

pub fn handle(opt_version: Option<String>, limit: String) {
    mem::check_if_valid(limit.clone());

    let manifest = get_manifest();
    let version = opt_version.unwrap_or(manifest.latest.snapshot.clone());

    let proj_dirs = ProjectDirs::from("me", "illia", "mc_cli").unwrap();
    let data_dir = proj_dirs.data_dir();
    let vers = data_dir.join("vers");
    let ver = vers.join(version.as_str());

    if ver.is_dir() {
        println!("Launching vanilla {} with memory limit {}", version, limit);

        launch(ver, limit.clone());

        return;
    }

    create_dirs(vers, ver.clone());
    
    let ver_url = get_ver_json_url(manifest, version.clone());

    let res = reqwest::blocking::get(&ver_url).expect("Failed to request version");
    let text = res.text().unwrap();

    fs::write(ver.join("version.json"), text).expect("Failed to write version json to file");
}
