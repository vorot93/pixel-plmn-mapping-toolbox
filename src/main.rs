use anyhow::Result;
use clap::Parser;
use pixel_plmn_mapping_toolbox::{
    cli::{Cli, Command},
    commands,
};

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Decode(io) => commands::decode(io.input.as_deref(), io.output.as_deref()),
        Command::Encode(io) => commands::encode(io.input.as_deref(), io.output.as_deref()),
        Command::InjectPlmn { plmn, mapping, io } => {
            commands::inject(&plmn, &mapping, io.input.as_deref(), io.output.as_deref())
        }
    }
}
