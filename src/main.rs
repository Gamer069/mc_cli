mod app;
mod vanilla;
mod mem;
mod fabric;

use clap::Parser;

fn main() {
    let app = app::App::parse();

    match app.command {
        app::Subcommand::Vanilla { version, mem } => {
            vanilla::handle(version, mem);
        },
        app::Subcommand::Fabric { version, loader_version, mem } => {
            fabric::handle(version, loader_version, mem);
        },
    }
}
