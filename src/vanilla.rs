use std::{borrow, clone, collections::HashMap, fs, io::{BufRead, BufReader}, path::{Path, PathBuf}, process::{Command, Stdio}};

use directories::ProjectDirs;
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


pub fn list_files_recursively(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                files.extend(list_files_recursively(&path));
            } else {
                files.push(path);
            }
        }
    }
    files
}

pub fn launch(json: VersionJson, version_dir: PathBuf, limit: String) {
    let game_dir = version_dir.parent().unwrap().parent().unwrap().join("game");
    let assets_dir = version_dir.parent().unwrap().parent().unwrap().join("assets");
    let libs = version_dir.join("libs");
    let mut classpath_paths = list_files_recursively(&libs);
    classpath_paths.push(version_dir.join("client.jar"));
    let classpath = if std::env::consts::OS == "windows" {
        classpath_paths.iter().map(|e| {
            e.to_string_lossy()
        }).collect::<Vec<_>>().join(";")
    } else {
        classpath_paths.iter().map(|e| {
            e.to_string_lossy()
        }).collect::<Vec<_>>().join(":")
    };
    let mut jvm_args: Vec<String> = vec![];
    jvm_args.push(format!("-Xmx{}", limit));
    // DONT MIND THIS CODE. I KNOW IT'S TERRIBLE
    if let Some(arguments) = json.arguments.clone() {
        for arg in arguments.jvm {
            match arg {
                version::JvmArgument::String(arg) => jvm_args.push(arg),
                version::JvmArgument::ArgWithRule { rules, value } => {
                    for rule in rules {
                        if rules::matches_os_rule(&rule) {
                            match value {
                                version::JvmArgumentValue::String(ref val) => {
                                    jvm_args.push(val.clone());
                                },
                                version::JvmArgumentValue::Strings(ref vals) => {
                                    for val in vals {
                                        jvm_args.push(val.clone());
                                    }
                                },
                            }
                        }
                    }
                },
            }
        }
    } else {
        jvm_args.push("-Djava.library.path=${natives_directory}".to_owned());
        jvm_args.push("-Djna.tmpdir=${natives_directory}".to_owned());
        jvm_args.push("-Dorg.lwjgl.system.SharedLibraryExtractPath=${natives_directory}".to_owned());
        jvm_args.push("-Dio.netty.native.workdir=${natives_directory}".to_owned());
        jvm_args.push("-Dminecraft.launcher.brand=${launcher_name}".to_owned());
        jvm_args.push("-Dminecraft.launcher.version=${launcher_version}".to_owned());
        jvm_args.push("-cp".to_owned());
        jvm_args.push("${classpath}".to_owned());
    }
    let jvm_args_str = jvm_args.join(" ")
        .replace("${natives_directory}", &libs.to_string_lossy().clone())
        .replace("${classpath}", &classpath)
        .replace("${launcher_name}", "mc_cli")
        .replace("${launcher_version}", env!("CARGO_PKG_VERSION"));

    let mut game_args: Vec<String> = vec![];
    let mut features: HashMap<String, bool> = HashMap::new();

    features.insert("is_demo_user".to_owned(), false);
    features.insert("has_custom_resolution".to_owned(), false);
    features.insert("has_quick_plays_support".to_owned(), false);
    features.insert("is_quick_play_singleplayer".to_owned(), false);
    features.insert("is_quick_play_multiplayer".to_owned(), false);
    features.insert("is_quick_play_realms".to_owned(), false);

    if let Some(arguments) = json.arguments.clone() {
        for arg in json.arguments.unwrap().game {
            match arg {
                version::GameArgument::String(arg) => game_args.push(arg),
                version::GameArgument::ArgWithRule { rules, value } => {
                    for rule in rules {
                        if rules::matches_arg_rule(features.clone(), &rule) {
                            match value {
                                version::GameArgumentValue::String(ref val) => {
                                    game_args.push(val.clone());
                                },
                                version::GameArgumentValue::Strings(ref vals) => {
                                    for val in vals {
                                        game_args.push(val.clone());
                                    }
                                },
                            }
                        }
                    }
                },
            }
        }
    } else {
        if let Some(minecraft_arguments) = json.minecraftArguments.clone() {
            let args = minecraft_arguments.split(" ").collect::<Vec<_>>();
            for arg in args {
                game_args.push(arg.to_owned());
            }
        }
    }
    let game_args_str = game_args
        .join(" ")
        .replace("${auth_player_name}", "qwerty")
        .replace("${version_name}", version_dir
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
        )
        .replace("${game_directory}", game_dir.to_str().unwrap())
        .replace("${auth_uuid}", &Uuid::new_v4().to_string())
        .replace("${auth_access_token}", &"".to_string())
        .replace("${clientid}", &Uuid::new_v4().to_string())
        .replace("${auth_xuid}", &"0".to_string())
        .replace("${user_type}", &"offline".to_string())
        .replace("${version_type}", &json.r#type)
        .replace("${assets_index_name}", &version_dir.file_name().unwrap().to_string_lossy())
        .replace("${assets_root}", assets_dir.to_str().unwrap());

    let mut cmd: Vec<String> = vec![];

    for jvm_arg in jvm_args_str.split(" ") {
        cmd.push(jvm_arg.to_owned());
    }

    cmd.push(json.mainClass);

    for game_arg in game_args_str.split(" ") {
        cmd.push(game_arg.to_owned());
    }

    println!("cmd: {:?}", cmd);

    let mut cmd = Command::new("java")
        .args(cmd)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run minecraft");

    let stdout = cmd.stdout.take().expect("Failed to take stdout");
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        let line = line.expect("Failed to get stdout line");
        println!("{}", line);
    }
    let status = cmd.wait().expect("Failed to wait for child to exit");
    println!("Exited with {}", status);
}

pub fn create_dirs(vers: PathBuf, ver: PathBuf) {
    let _ = fs::create_dir_all(vers.clone());
    let _ = fs::create_dir(ver.clone());
    let _ = fs::create_dir(vers.parent().unwrap().join("game"));
    let _ = fs::create_dir(ver.join("libs"));
    let _ = fs::create_dir(vers.parent().unwrap().join("assets"));
}

pub fn handle(opt_version: Option<String>, limit: String) {
    mem::check_if_valid(limit.clone());

    let manifest = get_manifest();
    let version = opt_version.unwrap_or(manifest.latest.snapshot.clone());

    let proj_dirs = ProjectDirs::from("me", "illia", "mc_cli").unwrap();
    let data_dir = proj_dirs.data_dir();
    let vers = data_dir.join("vers");
    let ver = vers.join(version.as_str());
    let libs = ver.join("libs");

    if ver.is_dir() {
        println!("Launching vanilla {} with memory limit {}", version, limit);

        let text = fs::read_to_string(ver.join("version.json")).expect("failed to read version.json");
        let mut version_json_err = serde_json::Deserializer::from_str(&text);
        let version_json_res = serde_path_to_error::deserialize::<_, VersionJson>(&mut version_json_err);
        let version_json = match version_json_res {
            Ok(val) => val,
            Err(err) => panic!("err: {:#?}", err),
        };

        launch(version_json, ver, limit.clone());

        return;
    }

    create_dirs(vers, ver.clone());

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
            if let Some(classifiers) = &lib.downloads.classifiers {
                let needed = rules::classifiers_needed(classifiers);

                println!("asdf needed: {:?}", &needed);

                for needed in needed {
                    println!("asdf url: {}", &needed.url);
                    let url = &needed.url;
                    let path = Path::new(&needed.path);
                    let _ = fs::create_dir_all(libs.join(path.parent().unwrap()));
                    let _ = util::download(needed.url.as_str(), &libs.as_path().join(path), "Downloaded classifier lib".to_owned()).expect("Failed to download classifier lib");
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

    launch(version_json, ver, limit.clone());
}
