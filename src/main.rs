#![allow(dead_code, unused_variables)]
mod version;
mod app;
mod vanilla;
mod mem;
mod fabric;
mod util;
mod rules;
mod assets;

use clap::Parser;
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
    }
}
