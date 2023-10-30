use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;

mod cmds {
    automod::dir!(pub "src/cmds");
}

/// CLI Tool to manage matches with RLBot
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Starts a match with RLBot
    Start(cmds::start::CommandArgs),
}

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();
    let args = Args::parse();

    match args.command {
        Commands::Start(x) => x.run(),
    }?;

    Ok(())
}
