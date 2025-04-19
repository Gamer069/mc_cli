use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "mc_cli", version = "0.0.1")]
pub struct App {
    #[clap(subcommand)]
    pub command: Subcommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum Subcommand {
    Vanilla {
        #[clap(short, long)]
        version: Option<String>,
        #[clap(short, long, default_value = "10G")]
        mem: String,
    },
    Fabric {
        #[clap(short, long)]
        version: Option<String>,
        #[clap(short, long)]
        loader_version: Option<String>,
        #[clap(short, long, default_value = "10G")]
        mem: String,
    },
}

