use crate::{util};

const FABRIC_GAME_VERSIONS: &'static str = "https://meta.fabricmc.net/v2/versions/game";

pub fn handle(opt_version: Option<String>, opt_loader_version: Option<String>, limit: String) {
    println!("Launching fabric {:?}-{:?} with memory limit {}", opt_version, opt_loader_version, limit);
    panic!("Fabric launching is currently still in development; any code AFTER this is a draft");
    let game_versions = util::download_text_no_save(FABRIC_GAME_VERSIONS, "Downloaded fabric manifest".to_owned());
}
