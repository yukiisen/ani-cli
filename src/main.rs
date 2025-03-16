use ani_core::database;
use ani_core::utils;
use anyhow::Result;
use clap::Parser;

mod command_map;
mod commands;

use command_map::Commands;
use command_map::CLI;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = CLI::parse();
    let config = utils::config::load_config()?;

    let pool = database::schema::initialize(&config).await?;

    match &cli.command {
        Commands::Info { id, details } => { commands::info::display_anime_info(id, *details, pool).await },
        _ => {}
    };

    Ok(())
}
