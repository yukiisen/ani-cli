# ani-cli

`ani-cli` is a command-line interface (CLI) tool for managing and retrieving anime data locally. It leverages the `ani-core` library for database interactions and provides a set of commands to search, update, and display anime information.

## Features

- **List**: List locally downloaded anime.
- **Update**: Update the current anime metadata and fetch any missing information.
- **Scan**: Scan local directories for any missing metadata.
- **Search**: Search for anime by keyword.
- **Info**: Fetch and display anime data using MAL ID.
- **Export**: Export all metadata to a specified path.
- **Add**: Add new anime entries to the local database.

## Installation

To install `ani-cli`, clone the repository and build the project using Cargo:

```sh
git clone https://github.com/yukiisen/ani-cli.git
cd ani-cli
cargo build --release
```
## Usage

The `ani-cli` tool provides several subcommands, each with its own set of options and arguments. Below is a brief overview of the available commands:

### Info

Fetch and display anime data using MAL ID.

```sh
ani-cli info <ID> [OPTIONS]

Options:
  -d, --details    Display detailed information.
```

## Configuration

The configuration file is located at `~/.config/ani-lib/config.json`. You can edit this file to change the database path, schema path, and other settings.

## TODO

- Implement the `Set` command to edit configuration.
- Implement the `List` command to list locally downloaded anime.
- Implement the `Update` command to fetch and update metadata.
- Implement the `Scan` command to scan local directories for missing metadata.
- Implement the `Search` command to search for anime by keyword.
- Implement the `Info` command to fetch anime data using MAL ID.
- Implement the `Export` command to export all metadata to a specified path.
- Implement the `Add` command to add new anime entries to the local database.
- Add more detailed error handling and user feedback.
- Extend the `main` function to handle all possible commands.

## Acknowledgements

- [ani-core](https://github.com/yukiisen/ani-core)
- [clap](https://clap.rs/)
- [sqlx](https://github.com/launchbadge/sqlx)
- [tokio](https://tokio.rs/)

**Note**: This project is still in development. Some features may not be fully implemented yet.
