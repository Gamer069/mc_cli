use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;

use crate::{mem, util};
use crate::version::{FabricLoaderVersion, FabricVersion};

const FABRIC_GAME_VERSIONS: &'static str = "https://meta.fabricmc.net/v2/versions/game";
const FABRIC_LOADER_VERSIONS: &'static str = "https://meta.fabricmc.net/v2/versions/loader";
const FABRIC_MAVEN: &'static str = "https://maven.fabricmc.net/";

pub fn get_ver(versions: Vec<FabricVersion>, version: String) -> FabricVersion {
    let mut found = false;
    let mut ver = &FabricVersion { version: "".to_string(), stable: false };

    versions.iter().any(|_ver| {
        found = _ver.version == version.clone();

        ver = _ver;

        found
    });
    ver.clone()
}

pub fn down(loader: &FabricLoaderVersion, version: &FabricVersion, ver: PathBuf) {
    let loader_jar_url = format!("{}{}", FABRIC_MAVEN, loader.jar_path());
    println!("downloading loader jar from {}", loader_jar_url);
    let _ = util::download(&loader_jar_url, ver.as_path(), "Downloaded fabric loader jar".to_string());
}

pub fn create_dirs(vers: PathBuf, ver: PathBuf) {
    let _ = fs::create_dir_all(vers.clone());
    let _ = fs::create_dir(ver.clone());
    let _ = fs::create_dir(vers.parent().unwrap().join("game"));
    let _ = fs::create_dir(ver.join("libs"));
    let _ = fs::create_dir(vers.parent().unwrap().join("assets"));
}

pub fn launch() {
}

pub fn handle(opt_version: Option<String>, opt_loader_version: Option<String>, limit: String) {
    let game_versions = util::download_text_no_save(FABRIC_GAME_VERSIONS, "Downloaded fabric game versions json".to_string()).expect("Failed to download fabric game versions json");
    let versions: Vec<FabricVersion> = serde_json::from_str(game_versions.as_str()).expect("Failed to parse fabric game versions JSON");
    let ver = if opt_version.is_some() {
        get_ver(versions, opt_version.unwrap())
    } else {
        versions.first().unwrap().clone()
    };

    let loader_versions_json = util::download_text_no_save(FABRIC_LOADER_VERSIONS, "".to_string()).expect("Failed to download loader versions JSON");
    let loader_versions: Vec<FabricLoaderVersion> = serde_json::from_str(&loader_versions_json).expect("Failed to parse fabric loader versions JSON");
    let loader = if let Some(ver_str) = opt_loader_version {
        &loader_versions
            .into_iter()
            .find(|v| v.version == ver_str)
            .expect("Loader version not found")
    } else {
        loader_versions.first().expect("No loader versions found")
    };

    let (loader_version, loader_build) = (loader.version.as_str(), loader.build);

    if !mem::is_valid(limit.clone()) {
        eprintln!("Invalid memory limit");
        std::process::exit(-1);
    }

    if !mem::can_use(limit.clone()) {
        eprintln!("Your memory limit is too big");
        std::process::exit(-1);
    }

    println!("Launching fabric {}-{} build {} with memory limit {}", ver.version, loader_version, loader_build, limit);

    let proj_dirs = ProjectDirs::from("me", "illia", "mc_cli").unwrap();
    let data_dir = proj_dirs.data_dir();
    let vers = data_dir.join("vers");
    let ver_path = vers.join(format!("fabric-{}", ver.version.clone()));

    create_dirs(vers, ver_path.clone());

    down(loader, &ver, ver_path.clone());

    launch();
}
