use clap::Parser;
use clap::Subcommand;

/// Does your paperwork
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub(crate) verbose: u8,

    #[arg(short, long)]
    pub(crate) quiet: bool,
}