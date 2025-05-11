use std::fs;
use std::io::{BufRead as _, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use directories::ProjectDirs;
use uuid::Uuid;

use crate::{mem, util, vanilla, version};
use crate::version::{FabricIntermediaryVersion, FabricLoaderJSON, FabricLoaderVersion, FabricVersion, UseQuilt};

const FABRIC_GAME_VERSIONS: &'static str = "https://meta.fabricmc.net/v2/versions/game";
const FABRIC_LOADER_VERSIONS: &'static str = "https://meta.fabricmc.net/v2/versions/loader";
const FABRIC_INTERMEDIARY_VERSIONS: &'static str = "https://meta.fabricmc.net/v2/versions/intermediary";
const FABRIC_MAVEN: &'static str = "https://maven.fabricmc.net/";

const QUILT_GAME_VERSIONS: &'static str = "https://meta.quiltmc.org/v3/versions/game";
const QUILT_LOADER_VERSIONS: &'static str = "https://meta.quiltmc.org/v3/versions/loader";
const QUILT_INTERMEDIARY_VERSIONS: &'static str = "https://meta.quiltmc.org/v3/versions/intermediary";
const QUILT_MAVEN: &'static str = "https://maven.quiltmc.org/";

pub fn get_ver(versions: Vec<FabricVersion>, version: String) -> FabricVersion {
    let mut found = false;
    let mut ver = &FabricVersion { version: "".to_string() };

    versions.iter().any(|_ver| {
        found = _ver.version == version.clone();

        ver = _ver;

        found
    });
    ver.clone()
}

pub async fn down_intermediary(loader: &FabricLoaderVersion, version: &FabricVersion, ver: PathBuf, use_quilt: UseQuilt) {
    let is_quilt = matches!(use_quilt, UseQuilt::Yes(_));
    let use_release = match use_quilt {
        UseQuilt::Yes(value) => value,
        UseQuilt::No => false,
    };
    let intermediary_versions_text = util::download_text_no_save_async(if is_quilt { QUILT_INTERMEDIARY_VERSIONS } else { FABRIC_INTERMEDIARY_VERSIONS }, "Downloaded intermediary version JSON".to_string()).await.expect("Failed to download intermediary version JSON");
    let intermediaries: Vec<FabricIntermediaryVersion> = serde_json::from_str(intermediary_versions_text.as_str()).expect("Failed to deserialize intermediary version JSON");
    let mut intermediary: &FabricIntermediaryVersion = &FabricIntermediaryVersion {
        maven: "".to_string(),
        version: "".to_string(),
    };
    intermediaries.iter().any(|i| {
        if i.version == version.version {
            intermediary = i;
            return true;
        }
        return false;
    });

    let maven_path = version::maven_to_path(intermediary.maven.clone());
    let maven_path_with_domain = format!("{}/{}", if is_quilt { format!("{}{}", QUILT_MAVEN, if use_release { "/release" } else { "/snapshot" }) } else { FABRIC_MAVEN.to_string() }, maven_path);
    let _ = util::download_async(maven_path_with_domain.as_str(), ver.join("inter.jar").as_ref(), "Downloaded intermediary...".to_string()).await.expect("Failed to download intermediary");
}

/// returns the full fabric loader JSON
/// {loader} the loader version
/// {version} the minecarft version
/// {ver} the version dir
pub async fn down(loader: &FabricLoaderVersion, version: &FabricVersion, ver: PathBuf, use_quilt: UseQuilt) -> FabricLoaderJSON {
    let is_quilt = matches!(use_quilt, UseQuilt::Yes(_));
    let use_release = match use_quilt {
        UseQuilt::Yes(value) => value,
        UseQuilt::No => false,
    };

    down_intermediary(loader, version, ver.clone(), use_quilt).await;
    let loader_jar_url = format!("{}{}", if is_quilt { QUILT_MAVEN } else { FABRIC_MAVEN }, loader.jar_path());
    println!("downloading loader jar from {}", loader_jar_url);

    let jar_path = ver.join("fabric.jar");

    if !jar_path.exists() {
        let _ = util::download_async(&loader_jar_url, jar_path.as_path(), "Downloaded fabric loader jar".to_string()).await.expect("Failed to download fabric JAR");
    }

    let loader_json_url = format!("{}{}", if is_quilt { QUILT_MAVEN } else { FABRIC_MAVEN }, loader.json_path());
    println!("Downloading loader JSON from {}", loader_json_url);

    let json_path = ver.join("fabric.json");

    let loader_json = if !json_path.exists() {
        util::download_text_async(&loader_json_url, json_path.as_path(), "Downloaded fabric loader JSON".to_string()).await.expect("Failed to download fabric loader JSON")
    } else {
        fs::read_to_string(json_path.as_path()).unwrap()
    };

    let parsed_json: FabricLoaderJSON = serde_json::from_str(&loader_json).expect("Failed to parse loader JSON");
    println!("{:#?}", parsed_json);

    for lib in &parsed_json.libraries.common {
        let path_from_maven = version::maven_to_path(lib.name.clone());
        let path = format!("{}{}", lib.url, path_from_maven);
        let lib_path = ver.join("libs").join(path_from_maven.clone());
        let _ = fs::create_dir_all(lib_path.clone().parent().unwrap());
        let _ = util::download_async(path.as_str(), &lib_path, "Downloaded common lib jar".to_owned()).await.expect("Failed to download server lib jar");
    }
    println!("Downloaded common libs...");
    for lib in &parsed_json.libraries.server {
        let path_from_maven = version::maven_to_path(lib.name.clone());
        let path = format!("{}{}", lib.url, path_from_maven);
        let lib_path = ver.join("libs").join(path_from_maven.clone());
        let _ = fs::create_dir_all(lib_path.clone().parent().unwrap());
        let _ = util::download_async(path.as_str(), &lib_path, "Downloaded server lib jar".to_owned()).await.expect("Failed ot download server lib jar");
    }
    println!("Downloaded server libs...");
    for lib in &parsed_json.libraries.client {
        let path_from_maven = version::maven_to_path(lib.name.clone());
        let path = format!("{}{}", lib.url, path_from_maven);
        let lib_path = ver.join("libs").join(path_from_maven.clone());
        let _ = fs::create_dir_all(lib_path.clone().parent().unwrap());
        let _ = util::download_async(path.as_str(), &lib_path, "Downloaded client lib jar".to_owned()).await.expect("Failed to download client lib jar");
    }
    println!("Downloaded client libs...");

    parsed_json
}

pub fn create_dirs(vers: PathBuf, ver: PathBuf) {
    let _ = fs::create_dir_all(vers.clone());
    let _ = fs::create_dir(ver.clone());
    let _ = fs::create_dir(vers.parent().unwrap().join("game"));
    let _ = fs::create_dir(ver.join("libs"));
    let _ = fs::create_dir(vers.parent().unwrap().join("assets"));
}

pub fn launch(ver_dir: PathBuf, main_class: String) {
    println!("Launching minecraft client...");
    let main_class = dbg!(main_class);
    let game_dir = ver_dir.join("game");
    let _ = fs::create_dir_all(&game_dir);

    let mut classpath: String = "".to_owned();
    let libs = util::list_files_recursively(&ver_dir.join("libs"));
    let sep = if cfg!(target_os = "windows") { ';' } else { ':' };

    for lib in libs {
        classpath.push_str(lib.to_str().unwrap());
        classpath.push(sep);
    }

    classpath.push_str(ver_dir.join("inter.jar").to_str().unwrap());
    classpath.push(sep);
    classpath.push_str(ver_dir.join("client.jar").to_str().unwrap());
    classpath.push(sep);
    classpath.push_str(ver_dir.join("fabric.jar").to_str().unwrap());

    println!("{}", classpath);

    let mut cmd: Vec<String> = vec![];
    cmd.push("-cp".to_owned());
    cmd.push(classpath);
    cmd.push(main_class);
    cmd.push("--gameDir".to_string());
    cmd.push(ver_dir.parent().unwrap().to_str().unwrap().to_string());
    cmd.push("--assetsDir".to_string());
    cmd.push(ver_dir.parent().unwrap().join("assets").to_str().unwrap().to_string());
    cmd.push("--assetIndex".to_string());
    let ver = ver_dir.file_name().unwrap().to_str().unwrap().replace("fabric-", "").replace("quilt-", "");
    cmd.push(ver);
    cmd.push("--uuid".to_string());
    cmd.push(Uuid::new_v4().to_string());

    println!("{:#?}", cmd);

    let mut process = Command::new("java")
        .current_dir(game_dir)
        .args(&cmd)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run Minecraft");

    let stdout = process.stdout.take().expect("Failed to take stdout");
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        let line = line.expect("Failed to read stdout line");
        println!("{}", line);
    }

    let status = process.wait().expect("Failed to wait for child");
    println!("Exited with {}", status);
}

pub async fn handle(opt_version: Option<String>, opt_loader_version: Option<String>, limit: String, use_quilt: UseQuilt) {
    let is_quilt = matches!(use_quilt, UseQuilt::Yes(_));
    let use_release = match use_quilt {
        UseQuilt::Yes(value) => value,
        UseQuilt::No => false,
    };
    let game_versions = util::download_text_no_save_async(if is_quilt { QUILT_GAME_VERSIONS } else { FABRIC_GAME_VERSIONS }, "Downloaded fabric game versions json".to_string()).await.expect("Failed to download fabric game versions json");
    let versions: Vec<FabricVersion> = serde_json::from_str(game_versions.as_str()).expect("Failed to parse fabric game versions JSON");
    let ver = if opt_version.is_some() {
        get_ver(versions, opt_version.unwrap())
    } else {
        versions.first().unwrap().clone()
    };

    let loader_versions_json = util::download_text_no_save_async(if is_quilt { QUILT_LOADER_VERSIONS } else { FABRIC_LOADER_VERSIONS }, "".to_string()).await.expect("Failed to download loader versions JSON");
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
    let ver_path = vers.join(format!("{}-{}", if is_quilt { "quilt" } else { "fabric" }, ver.version.clone()));

    create_dirs(vers, ver_path.clone());

    vanilla::handle(Some(ver.version.clone()), limit.clone(), false, Some(ver_path.as_path())).await;
    let parsed_json = down(loader, &ver, ver_path.clone(), use_quilt).await;

    launch(ver_path, parsed_json.mainClass.client);
}
