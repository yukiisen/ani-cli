use clap::{ Parser, Subcommand };
use std::path::PathBuf;

#[derive(Parser)]
#[clap(author, version, about = "AnimeLib Cli by @yukiisen, Check the repository for more information.")]
pub struct CLI {
    #[command(subcommand)]
    pub command: Commands
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "List locally downloaded anime.")]
    List {},

    #[command(about = "Update the current anime metadata and fetch anything messing.")]
    Update {
        #[arg(short, long, default_value_t = false)]
        #[arg(help = "Refetch The whole library.")]
        full: bool,

        #[arg(short, long, default_value_t = false)]
        #[arg(help = "Log steps and API responses.")]
        verbose: bool,
    },

    #[command(about = "Scans local Directories for any missing metadata, Use the `update` command to fetch them afterwards.")]
    Scan {},

    #[command(about = "Search for anime with <keyword>")]
    Search {
        #[arg(help = "keyword to search for.")]
        keyword: String,

        #[arg(help = "Search in local database only.")]
        #[arg(short, long, default_value_t = false)]
        local: bool
    },

    #[command(about = "Locally fetch anime data using mal_id")]
    Info {
        #[arg(help = "Anime MAL-ID.")]
        id: u32,

        #[arg(help = "Display details?")]
        #[arg(short, long, default_value_t = false)]
        details: bool,
    },

    #[command(about = "Export all metadata to <PATH>")]
    Export {
        #[arg(help = "Target path to export data.")]
        path: PathBuf
    },

    #[command(about = "Edit configuration.")]
    Set {
        // TODO: Shall be implemented later.
    },

    #[command(about = "Add new animes to the list, use `update` afterwards to fetch metadata.")]
    Add {
        #[arg(help = "Anime MAL-ID")]
        #[arg(short, long, required = true)]
        id: Vec<u32>,

        #[arg(help = "Anime name")]
        #[arg(short, long)]
        name: Vec<String>
    }
}
