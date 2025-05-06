#![allow(dead_code, unused_variables)]
mod version;
mod app;
mod vanilla;
mod mem;
mod fabric;
mod util;
mod rules;
mod assets;
mod auth;

use clap::Parser;
use dotenv::dotenv;

fn main() {
    let app = app::App::parse();

    match app.command {
        app::Subcommand::Vanilla { version, mem } => {
            dotenv().ok();

            let cli = std::env::var("CLI").expect("Client ID not in .env");

            let (xid, jwt, uuid, name) = auth::authenticate(cli.clone());

            vanilla::handle(version, mem, xid, jwt, cli, uuid, name);
        },
        app::Subcommand::Fabric { version, loader_version, mem } => {
            fabric::handle(version, loader_version, mem);
        },
    }
}
