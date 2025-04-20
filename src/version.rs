use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Arguments {
    pub game: Vec<GameArgument>,
    pub jvm: Vec<JvmArgument>,
}

#[derive(Deserialize, Debug)]
pub struct Download {
    pub sha1: String,
    pub size: i32,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct LibraryDownload {
    pub path: String,
    pub sha1: String,
    pub size: i32,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct Downloads {
    pub client: Download,
    pub client_mappings: Download,
    pub server: Download,
    pub server_mappings: Download,
}

#[derive(Deserialize, Debug)]
pub struct LibraryDownloads {
    pub artifact: LibraryDownload,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Os {
    Name { name: String },
    Arch { arch: String},
    Both { name: String, arch: String },
    // pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct OsRule {
    pub action: String,
    pub os: Os,
}

#[derive(Deserialize, Debug)]
// because serde doesn't wanna rename it for me!
#[allow(non_snake_case)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: i32,
    pub totalSize: i32,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct Library {
    pub downloads: LibraryDownloads,
    pub name: String,
    pub rules: Option<Vec<OsRule>>,
}

#[derive(Deserialize, Debug)]
// because serde doesn't wanna rename it for me!
#[allow(non_snake_case)]
pub struct VersionJson {
    pub arguments: Arguments,
    pub downloads: Downloads,
    pub libraries: Vec<Library>,
    pub mainClass: String,
    pub r#type: String,
    pub assetIndex: AssetIndex,
}

#[derive(Deserialize, Debug)]
pub struct ArgRuleFeatures(pub HashMap<String, bool>);

#[derive(Deserialize, Debug)]
pub struct ArgRule {
    pub action: String,
    pub features: ArgRuleFeatures,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum GameArgumentValue {
    String(String),
    Strings(Vec<String>),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum JvmArgumentValue {
    String(String),
    Strings(Vec<String>),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum JvmArgument {
    String(String),
    ArgWithRule { rules: Vec<OsRule>, value: JvmArgumentValue },
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum GameArgument {
    String(String),
    ArgWithRule { rules: Vec<ArgRule>, value: GameArgumentValue },
}
