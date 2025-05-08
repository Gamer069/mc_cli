use std::{borrow, clone, collections::HashMap, fs, io::{BufRead, BufReader}, path::{Path, PathBuf}, process::{Command, Stdio}};

use directories::ProjectDirs;
use jars::{Jar, JarOptionBuilder};
use serde::Deserialize;
use uuid::Uuid;

use crate::{assets::AssetIndexJson, mem, rules, util, version::{self, VersionJson}};

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
    let manifest_txt = util::download_text_no_save(VANILLA_MANIFEST, "Downloaded vanilla manifest".to_owned()).expect("Failed to download vanilla manifest to RAM");
    serde_json::from_str(&manifest_txt).unwrap()
}


pub fn launch(json: VersionJson, version_dir: PathBuf, limit: String) {
    let game_dir = version_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("game");
    let assets_dir = version_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("assets");
    let libs = version_dir.join("libs");

    let mut classpath_paths = util::list_files_recursively(&libs);
    classpath_paths.push(version_dir.join("client.jar"));
    let classpath = if std::env::consts::OS == "windows" {
        classpath_paths
            .iter()
            .map(|e| e.to_string_lossy())
            .collect::<Vec<_>>()
            .join(";")
    } else {
        classpath_paths
            .iter()
            .map(|e| e.to_string_lossy())
            .collect::<Vec<_>>()
            .join(":")
    };

    let mut jvm_args: Vec<String> = vec![format!("-Xmx{}", limit)];

    if let Some(arguments) = json.arguments.clone() {
        for arg in arguments.jvm {
            match arg {
                version::JvmArgument::String(arg) => jvm_args.push(arg),
                version::JvmArgument::ArgWithRule { rules, value } => {
                    for rule in &rules {
                        if rules::matches_os_rule(rule) {
                            match value {
                                version::JvmArgumentValue::String(ref val) => {
                                    jvm_args.push(val.clone());
                                }
                                version::JvmArgumentValue::Strings(ref vals) => {
                                    jvm_args.extend(vals.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        jvm_args.extend(vec![
            "-Djava.library.path=${natives_directory}".to_owned(),
            "-Djna.tmpdir=${natives_directory}".to_owned(),
            "-Dorg.lwjgl.system.SharedLibraryExtractPath=${natives_directory}".to_owned(),
            "-Dio.netty.native.workdir=${natives_directory}".to_owned(),
            "-Dminecraft.launcher.brand=${launcher_name}".to_owned(),
            "-Dminecraft.launcher.version=${launcher_version}".to_owned(),
            "-cp".to_owned(),
            "${classpath}".to_owned(),
        ]);
    }

    let jvm_args_resolved: Vec<String> = jvm_args
        .into_iter()
        .map(|arg| {
            arg.replace("${natives_directory}", &libs.to_string_lossy())
                .replace("${classpath}", &classpath)
                .replace("${launcher_name}", "mc_cli")
                .replace("${launcher_version}", env!("CARGO_PKG_VERSION"))
        })
        .collect::<Vec<_>>();

    let mut game_args: Vec<String> = vec![];
    let mut features: HashMap<String, bool> = HashMap::new();

    features.insert("is_demo_user".to_owned(), false);
    features.insert("has_custom_resolution".to_owned(), false);
    features.insert("has_quick_plays_support".to_owned(), false);
    features.insert("is_quick_play_singleplayer".to_owned(), false);
    features.insert("is_quick_play_multiplayer".to_owned(), false);
    features.insert("is_quick_play_realms".to_owned(), false);

    if let Some(arguments) = json.arguments.clone() {
        for arg in arguments.game {
            match arg {
                version::GameArgument::String(arg) => game_args.push(arg),
                version::GameArgument::ArgWithRule { rules, value } => {
                    for rule in &rules {
                        if rules::matches_arg_rule(features.clone(), rule) {
                            match value {
                                version::GameArgumentValue::String(ref val) => {
                                    game_args.push(val.clone());
                                }
                                version::GameArgumentValue::Strings(ref vals) => {
                                    game_args.extend(vals.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    } else if let Some(minecraft_arguments) = json.minecraftArguments.clone() {
        let args = minecraft_arguments.split(' ').map(|s| s.to_owned()).collect::<Vec<_>>();
        game_args.extend(args);
    }

    let game_args_resolved: Vec<String> = game_args
        .into_iter()
        .map(|arg| {
            arg.replace("${auth_player_name}", "qwerty")
                .replace(
                    "${version_name}",
                    version_dir.file_name().unwrap().to_str().unwrap(),
                )
                .replace("${game_directory}", game_dir.to_str().unwrap())
                .replace("${auth_uuid}", &Uuid::new_v4().to_string())
                .replace("${auth_access_token}", "")
                .replace("${clientid}", &Uuid::new_v4().to_string())
                .replace("${auth_xuid}", "0")
                .replace("${user_type}", "offline")
                .replace("${version_type}", &json.r#type)
                .replace("${user_properties}", "{}")
                .replace(
                    "${assets_index_name}",
                    &version_dir.file_name().unwrap().to_string_lossy(),
                )
                .replace("${assets_root}", assets_dir.to_str().unwrap())
                .replace("${game_assets}", assets_dir.to_str().unwrap())
        })
        .collect::<Vec<_>>();

    let mut cmd: Vec<String> = vec![];
    cmd.extend(jvm_args_resolved);
    cmd.push(json.mainClass);
    cmd.extend(game_args_resolved);

    println!("cmd: {:?}", cmd);


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

pub fn create_dirs(vers: PathBuf, ver: PathBuf) {
    let _ = fs::create_dir_all(vers.clone());
    let _ = fs::create_dir(ver.clone());
    let _ = fs::create_dir(vers.parent().unwrap().join("game"));
    let _ = fs::create_dir(ver.join("libs"));
    let _ = fs::create_dir(vers.parent().unwrap().join("assets"));
}

pub fn handle(opt_version: Option<String>, limit: String, b_launch: bool, version_dir: Option<&Path>) {
    mem::check_if_valid(limit.clone());

    let manifest = get_manifest();
    let version = opt_version.unwrap_or(manifest.latest.snapshot.clone());

    let proj_dirs = ProjectDirs::from("me", "illia", "mc_cli").unwrap();
    let data_dir = proj_dirs.data_dir();
    let vers = data_dir.join("vers");
    let binding = vers.join(version.as_str());
    let ver = version_dir.unwrap_or(&binding);
    let libs = ver.join("libs");

    'launch_logic: {
        if ver.is_dir() {
            println!("Launching vanilla {} with memory limit {}", version, limit);

            let text = fs::read_to_string(ver.join("version.json"));
            if !text.is_ok() {
                break 'launch_logic;
            }
            let text = text.unwrap();
            let mut version_json_err = serde_json::Deserializer::from_str(&text);
            let version_json_res = serde_path_to_error::deserialize::<_, VersionJson>(&mut version_json_err);
            let version_json = match version_json_res {
                Ok(val) => val,
                Err(err) => panic!("err: {:#?}", err),
            };

            if b_launch {
                launch(version_json, ver.to_path_buf(), limit.clone());
            }

            return;
        }
    }

    create_dirs(vers, ver.to_path_buf());

    let ver_url = get_ver_json_url(manifest, version.clone());

    let text = util::download_text(ver_url.as_str(), ver.join("version.json").as_path(), "Downloaded version.json".to_owned()).expect("Failed to download version json");

    let mut version_json_err = serde_json::Deserializer::from_str(text.as_str());
    let version_json_res = serde_path_to_error::deserialize::<_, VersionJson>(&mut version_json_err);
    let version_json = match version_json_res {
        Ok(val) => val,
        Err(err) => panic!("err: {:#?}", err),
    };

    // download minecraft jar
    let client_url = version_json.downloads.client.url.clone();
    let _ = util::download(client_url.as_str(), ver.join("client.jar").as_path(), "Downloaded client jar".to_owned()).expect("Failed to download client jar");

    for lib in &version_json.libraries {
        if let Some(rules) = &lib.rules {
            for rule in rules {
                if let Some(artifact) = &lib.downloads.artifact {
                    let matches_rule = rules::matches_os_rule(rule);
                    if matches_rule {
                        let path = Path::new(&artifact.path);
                        let _ = fs::create_dir_all(libs.join(path.parent().unwrap()));
                        let _ = util::download(artifact.url.as_str(), &libs.as_path().join(path), "Downloaded lib".to_owned()).expect("Failed to download library");
                    }
                }
            }
        } else {
            if let Some(artifact) = &lib.downloads.artifact {
                let path = Path::new(&artifact.path);
                let _ = fs::create_dir_all(libs.join(path.parent().unwrap()));
                let _ = util::download(artifact.url.as_str(), &libs.as_path().join(path), "Downloaded lib".to_owned()).expect("Failed to download library");
            }
        }
        if let Some(classifiers) = &lib.downloads.classifiers {
            let needed = rules::classifiers_needed(classifiers);

            println!("asdf needed: {:?}", &needed);

            for needed in needed {
                println!("asdf url: {}", &needed.url);
                let url = &needed.url;
                let path = Path::new(&needed.path);
                let _ = fs::create_dir_all(libs.join(path.parent().unwrap()));
                let _ = util::download(needed.url.as_str(), &libs.as_path().join(path), "Downloaded classifier lib".to_owned()).expect("Failed to download classifier lib");
                if let Some(extract) = &lib.extract {
                    let excluded = extract.clone().exclude;
                    let option = JarOptionBuilder::builder().target(libs.to_str().unwrap()).build();
                    let lib = libs.as_path().join(path);
                    println!("lib: {:#?}", lib);
                    let jar = jars::jar(lib, option).expect("Failed to extract library jar file");
                    for file in jar.files {
                        let dir = if Path::new(&file.0).is_dir() {
                            Path::new(&file.0)
                        } else {
                            Path::new(&file.0).parent().unwrap()
                        };
                        fs::create_dir_all(libs.join(dir)).expect("Failed to create_dir_all before extracting library file");
                        fs::write(libs.join(file.0.clone()), file.1).expect(&format!("Failed to write extracted library file {}", file.0));
                    }
                    for excluded in excluded {
                        let _ = fs::remove_file(excluded); // don't care if it errors or not
                    }
                }
            }
        }
    }
    let assets_dir = data_dir.join("assets");
    let _ = fs::create_dir(assets_dir.join("indexes"));

    let asset_index_url = version_json.assetIndex.url.clone();
    let asset_index = util::download_text(&asset_index_url, &assets_dir.join("indexes").join(format!("{}.json", version)), "Downloaded asset index".to_owned()).expect("Failed to download asset index json");

    let asset_index_json: AssetIndexJson = serde_json::from_str(&asset_index).expect("Failed to parse asset index json");
    let assets = asset_index_json.objects;

    for asset in assets.iter() {
        let hash = &asset.1.hash;
        let dir = &hash[..2];
        let dir_full = assets_dir.join("objects").join(dir);
        let _ = fs::create_dir_all(&dir_full);
        util::download(&format!("https://resources.download.minecraft.net/{}/{}", dir, hash).to_string(), &dir_full.join(hash), "Downloaded resources".to_owned()).expect(format!("Failed to download resource {}", hash).as_str());
    }

    if b_launch {
        launch(version_json, ver.to_path_buf(), limit.clone());
    }
}
