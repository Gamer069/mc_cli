use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
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
    pub client_mappings: Option<Download>,
    pub server: Option<Download>,
    pub server_mappings: Option<Download>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct LibraryClassifiers {
    pub natives_windows_64: Option<LibraryDownload>,
    pub natives_windows_32: Option<LibraryDownload>,
    pub natives_windows: Option<LibraryDownload>,
    pub natives_osx: Option<LibraryDownload>,
    pub natives_osx_64: Option<LibraryDownload>,
    pub natives_osx_32: Option<LibraryDownload>,
    pub natives_linux: Option<LibraryDownload>,
    pub natives_linux_32: Option<LibraryDownload>,
    pub natives_linux_64: Option<LibraryDownload>,
}

#[derive(Deserialize, Debug)]
pub struct LibraryDownloads {
    pub artifact: Option<LibraryDownload>,
    pub classifiers: Option<LibraryClassifiers>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Os {
    Name { name: String },
    Arch { arch: String},
    Both { name: String, arch: String },
    // pub name: String,
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

#[derive(Deserialize, Debug, Clone)]
pub struct Extract {
    pub exclude: Vec<String>
}

#[derive(Deserialize, Debug)]
pub struct Library {
    pub downloads: LibraryDownloads,
    pub name: String,
    pub rules: Option<Vec<Rule>>,
    pub extract: Option<Extract>,
}

#[derive(Deserialize, Debug)]
// because serde doesn't wanna rename it for me!
#[allow(non_snake_case)]
pub struct VersionJson {
    pub arguments: Option<Arguments>,
    pub minecraftArguments: Option<String>,
    pub downloads: Downloads,
    pub libraries: Vec<Library>,
    pub mainClass: String,
    pub r#type: String,
    pub assetIndex: AssetIndex,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RuleFeatures(pub HashMap<String, bool>);

#[derive(Deserialize, Debug, Clone)]
pub struct Rule {
    pub action: String,
    pub features: Option<RuleFeatures>,
    pub os: Option<Os>
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum GameArgumentValue {
    String(String),
    Strings(Vec<String>),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum JvmArgumentValue {
    String(String),
    Strings(Vec<String>),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum JvmArgument {
    String(String),
    ArgWithRule { rules: Vec<Rule>, value: JvmArgumentValue },
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum GameArgument {
    String(String),
    ArgWithRule { rules: Vec<Rule>, value: GameArgumentValue },
}
