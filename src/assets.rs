use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AssetIndexJson {
    pub objects: HashMap<String, Object>
}

#[derive(Deserialize, Debug)]
pub struct Object {
    pub hash: String,
    pub size: i32,
}
