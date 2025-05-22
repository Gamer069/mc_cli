#![allow(dead_code, unused_variables)]
mod version;
mod app;
mod vanilla;
mod mem;
mod fabric;
mod util;
mod rules;
mod assets;
mod liteloader;

use app::OpenTarget;
use clap::Parser;
use cli_table::{Cell as _, Table};
use version::UseQuilt;

#[tokio::main]
async fn main() {
    let app = app::App::parse();

    match app.command {
        app::Subcommand::Vanilla { version, mem } => {
            vanilla::handle(version, mem, true, None).await;
        },
        app::Subcommand::Fabric { version, loader_version, mem } => {
            fabric::handle(version, loader_version, mem, UseQuilt::No).await;
        },
        app::Subcommand::Quilt { version, loader_version, mem, use_release } => {
            fabric::handle(version, loader_version, mem, UseQuilt::Yes(use_release)).await;
        },
        app::Subcommand::Liteloader { version, loader_version, mem } => {
            eprintln!("Liteloader isn't implemented yet. Please consider using fabric,quilt,or just running vanilla");
            liteloader::handle(version, loader_version, mem).await;
        },
        app::Subcommand::Open { target: OpenTarget::Game } => {
            let dirs = directories::ProjectDirs::from("me", "illia", "mc_cli").unwrap();
            let path = dirs.data_dir().join("game");
            open::that(path).unwrap();
        },
        app::Subcommand::Open { target: OpenTarget::Mods } => {
            let dirs = directories::ProjectDirs::from("me", "illia", "mc_cli").unwrap();
            let path = dirs.data_dir().join("game").join("mods");
            open::that(path).unwrap();
        },
        app::Subcommand::Open { target: OpenTarget::ResourcePacks } => {
            let dirs = directories::ProjectDirs::from("me", "illia", "mc_cli").unwrap();
            let path = dirs.data_dir().join("game").join("resourcepacks");
            open::that(path).unwrap();
        },
        app::Subcommand::Open { target: OpenTarget::Saves } => {
            let dirs = directories::ProjectDirs::from("me", "illia", "mc_cli").unwrap();
            let path = dirs.data_dir().join("game").join("saves");
            open::that(path).unwrap();
        },
        app::Subcommand::Open { target: OpenTarget::Logs } => {
            let dirs = directories::ProjectDirs::from("me", "illia", "mc_cli").unwrap();
            let path = dirs.data_dir().join("game").join("logs");
            open::that(path).unwrap();
        },
        app::Subcommand::Open { target: OpenTarget::Downloads } => {
            let dirs = directories::ProjectDirs::from("me", "illia", "mc_cli").unwrap();
            let path = dirs.data_dir().join("game").join("downloads");
            open::that(path).unwrap();
        },
        app::Subcommand::Open { target: OpenTarget::Data } => {
            let dirs = directories::ProjectDirs::from("me", "illia", "mc_cli").unwrap();
            let path = dirs.data_dir().join("game").join("data");
            open::that(path).unwrap();
        },
        app::Subcommand::Open { target: OpenTarget::Config } => {
            let dirs = directories::ProjectDirs::from("me", "illia", "mc_cli").unwrap();
            let path = dirs.data_dir().join("game").join("config");
            open::that(path).unwrap();
        },
        app::Subcommand::Open { target: OpenTarget::McOptions } => {
            let dirs = directories::ProjectDirs::from("me", "illia", "mc_cli").unwrap();
            let path = dirs.data_dir().join("game").join("options.txt");
            open::that(path).unwrap();
        },
        app::Subcommand::Versions => {
            let dirs = directories::ProjectDirs::from("me", "illia", "mc_cli").unwrap();
            let path = dirs.data_dir().join("vers");

            println!("Installed versions:");

            let mut rows = vec![
                vec![
                    "TYPE".cell(),
                    "NAME".cell()
                ]
            ];

            // Then add directories
            for dir_entry in path.read_dir().unwrap() {
                let path = dir_entry.unwrap().path();
                let fname = path.file_name().unwrap().to_str().unwrap();

                let (type_name, colored_type) = if fname.starts_with("fabric-") {
                    ("fabric", "fabric")
                } else if fname.starts_with("liteloader-") {
                    ("liteloader", "liteloader")
                } else if fname.starts_with("quilt-") {
                    ("quilt", "quilt")
                } else {
                    ("vanilla", "vanilla")
                };

                let name = fname.strip_prefix(&format!("{}-", type_name)).unwrap_or(fname);

                rows.push(vec![
                    colored_type.cell(),
                    name.cell(),
                ]);
            }

            // Create table and print
            let table = rows.table();
            println!("{}", table.display().unwrap());
        },
    }
}
