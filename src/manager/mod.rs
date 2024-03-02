use clap::{Parser, Subcommand};
use tracing::debug;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Name of the person to greet
    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[clap(subcommand)]
    Read(Entity),
    #[clap(subcommand)]
    Delete(Entity),
}

#[derive(Debug, Clone, Subcommand)]
pub enum Entity {
    User { id: i32 },
    Song { id: i32 },
    Score { id: i32 },
}

pub fn parse_command(command: &Command) -> anyhow::Result<()> {
    match command {
        Command::Read(entity) => {
            debug!("Read command input: {:?}", entity);
            Err(anyhow::anyhow!("placeholder"))
        }
        Command::Delete(entity) => {
            debug!("Delete command input: {:?}", entity);
            Ok(())
        }
    }
}
