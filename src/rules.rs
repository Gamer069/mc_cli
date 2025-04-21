use std::collections::HashMap;

use crate::version::{LibraryClassifiers, LibraryDownload, Rule};

pub fn matches_os_rule(rule: &Rule) -> bool {
    let mut matches_rule: bool = false;
    let os = &rule.os;
    if os.is_none() {
        return true;
    }
    match os.clone().unwrap().clone() {
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

pub fn matches_arg_rule(features: HashMap<String, bool>, rule: &Rule) -> bool {
    if rule.features.is_none() {
        return true;
    }
    let matches_rule: bool = hashmap_contains::<String, bool>(&features, &rule.features.clone().unwrap().0);
    return matches_rule && rule.action == "allow";
}

pub fn classifiers_needed(classifiers: &LibraryClassifiers) -> Vec<&LibraryDownload> {
    let mut downloads = vec![];

    let arch = std::env::consts::ARCH; // This is better than target_pointer_width.

    match std::env::consts::OS {
        "macos" => {
            if let Some(download) = &classifiers.natives_osx {
                downloads.push(download);
            }
            if let Some(download) = &classifiers.natives_osx_64 {
                downloads.push(download);
            }
            if let Some(download) = &classifiers.natives_osx_32 {
                downloads.push(download);
            }
        }
        "windows" => {
            if arch == "x86_64" {
                if let Some(download) = &classifiers.natives_windows_64 {
                    downloads.push(download);
                }
            } else if arch == "x86" {
                if let Some(download) = &classifiers.natives_windows_32 {
                    downloads.push(download);
                }
            }

            if let Some(download) = &classifiers.natives_windows {
                downloads.push(download);
            }
        }
        "linux" => {
            if arch == "x86_64" {
                if let Some(download) = &classifiers.natives_linux_64 {
                    downloads.push(download);
                }
            } else if arch == "x86" {
                if let Some(download) = &classifiers.natives_linux_32 {
                    downloads.push(download);
                }
            }

            if let Some(download) = &classifiers.natives_linux {
                downloads.push(download);
            }
        }
        _ => {}
    }

    downloads
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
