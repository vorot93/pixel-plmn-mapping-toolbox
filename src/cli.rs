use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "pixel-plmn-mapping-editor", version)]
#[command(about = "Inspect and edit the Pixel ap_plmn_mapping.binarypb")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Decode a binarypb to TOML.
    Decode(Io),
    /// Encode TOML to a binarypb.
    Encode(Io),
    /// Append a PLMN (MCC-MNC) to an existing mapping, by name.
    InjectPlmn {
        /// PLMN as MCC-MNC, e.g. 250-01.
        plmn: String,
        /// Target mapping name (identifier).
        mapping: String,
        #[command(flatten)]
        io: Io,
    },
}

#[derive(Args)]
pub struct Io {
    /// Input file (default: stdin).
    #[arg(short = 'i', long = "in")]
    pub input: Option<PathBuf>,
    /// Output file (default: stdout).
    #[arg(short = 'o', long = "out")]
    pub output: Option<PathBuf>,
}
