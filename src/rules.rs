use std::collections::HashMap;

use crate::version::{ArgRule, OsRule};

pub fn matches_os_rule(rule: &OsRule) -> bool {
    let mut matches_rule: bool = false;
    match &rule.os {
        crate::version::Os::Name { name } => {
            let os = rust_os_to_minecraft_os();
            if name == os {
                matches_rule = true;
            }
        },
        crate::version::Os::Arch { arch } => {
            let cur_arch = rust_arch_to_minecraft_arch();
            if arch == cur_arch {
                matches_rule = true;
            }
        },
        crate::version::Os::Both { name, arch } => {
            let cur_arch = rust_arch_to_minecraft_arch();
            if arch == cur_arch {
                matches_rule = true;
            }
            let os = rust_os_to_minecraft_os();
            if name == os && matches_rule {
                matches_rule = true;
            }
        },
    }
    return matches_rule && rule.action == "allow";
}

pub fn matches_arg_rule(features: HashMap<String, bool>, rule: &ArgRule) -> bool {
    let matches_rule: bool = hashmap_contains::<String, bool>(&features, &rule.features.0);
    return matches_rule && rule.action == "allow";
}

pub fn hashmap_contains<K: Eq + std::hash::Hash, V: PartialEq>(
    big: &std::collections::HashMap<K, V>,
    small: &std::collections::HashMap<K, V>,
) -> bool {
    small.iter().all(|(k, v)| big.get(k) == Some(v))
}

pub fn rust_os_to_minecraft_os() -> &'static str {
    match std::env::consts::OS {
        "windows" => "windows",
        "macos" => "osx",
        "linux" => "linux",
        _ => "unknown", // fallback for weird platforms
    }
}
pub fn rust_arch_to_minecraft_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "x86" => "x86",
        "aarch64" => "aarch64",
        "arm" => "arm32",
        _ => "unknown",
    }
}
