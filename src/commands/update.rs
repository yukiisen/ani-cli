#![allow(unused)]
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use indicatif::ProgressStyle;
use indicatif::ProgressBar;

use sqlx::SqlitePool;
use sqlx::FromRow;

use tokio::fs;
use tokio::time::sleep;

use reqwest::get;

use anyhow::Result;
use anyhow::Context;

use jikan_rs::anime::search_anime;
use jikan_rs::models::anime::Anime;

use ani_core::{
    database::queries, 
    utils::config::Config
};

pub async fn update_database (full: &bool, verbose: &bool, config: &Config, pool: &SqlitePool) -> Result<()> {
    // Dev-notes:
    // - when full is false, target_animes will only contain localNames.
    // - when full is false, target_animes is smaller the local_animes.
    // - when full is true, target_animes will contain db names and localNames.
    // - here target_animes is the same size as local_animes.
    // - also localNames and target_animes and results have the same ordering.

    let mut local_animes = fs::read_dir(&config.animes_path).await?;
    let mut local_entries = Vec::new();
    let mut target_animes = Vec::new();
    
    while let Some(entry) = local_animes.next_entry().await? {
        if entry.path().is_file() { continue; };
        let Ok(name) = entry.file_name().into_string() else { continue; };

        local_entries.push(name.clone());
        
        let name = if let Some(n) = get_anime_name(&name, pool).await? {
            if *full {
                n
            } else {
                continue;
            }
        } else {
            name
        };

        target_animes.push(name);
    }

    let spinner = ProgressBar::new_spinner()
        .with_style(
            ProgressStyle::with_template("{spinner} {msg}")?
        );

    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner.set_message("Updating Database...");

    let mut results = Vec::new(); // and array with anime entries. (localName, Anime)

    // collect anime data.
    for (idx, anime_name) in target_animes.iter().enumerate() {
        spinner.set_message(format!("Fetching {anime_name}"));

        // Skip on error since i couldn't identify the weird error I'm getting.
        let first_result =  match search_anime(anime_name, Some(1), None).await {
            Ok(result) => result.data,
            Err(error) => {
                spinner.println(format!("Failed to fetch {anime_name}.."));
                spinner.println(format!("{error}"));
                continue;
            }
        };

        let Some(res) = first_result.first().cloned() else {
            spinner.println(format!("Failed to fetch {anime_name}."));
            continue;
        };

        if res.title.to_lowercase() == *anime_name.to_lowercase() { 
            if *verbose { spinner.println(format!("Found match for {anime_name}.")); };
            results.push(({ if *full { &local_entries[idx] } else { anime_name } }, res));
            add_or_update_anime(results.last().unwrap(), pool).await?;
            if *verbose { spinner.println(format!("Saved {}.", results.last().unwrap().0)); };
        }
        else {
			let more_results = match search_anime(anime_name, Some(5), None).await {
			    Ok(result) => result.data,
			    Err(error) => {
			        spinner.println(format!("Failed to fetch {anime_name}.."));
                    spinner.println(format!("{error}"));
			        continue;
			    }
			};

            let found = spinner.suspend(|| {
                println!("Could not match {anime_name}, please pick it manually:");
                for (i, res) in more_results.iter().enumerate() {
                    println!("{}) {}.", i + 1, res.title);
                }

                loop {
                    print!("Pick one (Default 1, use 0 for none): ");
                    let mut buf = String::new();
                    std::io::stdout().flush().unwrap();
                    std::io::stdin().read_line(&mut buf).unwrap();

                    if buf.trim().is_empty() {
                        buf = "1".to_string();
                    };

                    let Ok(index)= buf.trim().parse::<u8>() else {
                        continue;
                    };

                    if index == 0 {
                        return None;
                    };

                    if let Some(anime) = more_results.get(index as usize - 1).cloned() {
                        return Some(anime);
                    };
                };
            });

            if let Some(anime) = found {
                if *verbose { spinner.println(format!("Found result for {anime_name}")); };
                
                if *full {
                    let local_name = &local_entries[idx];
                    results.push((local_name, anime));
                    add_or_update_anime(results.last().unwrap(), pool).await?;
                    if *verbose { spinner.println(format!("Saved {}.", local_name)); };
                } else {
                    results.push((anime_name, anime));
                    add_or_update_anime(results.last().unwrap(), pool).await?;
                    if *verbose { spinner.println(format!("Saved {}.", anime_name)); };
                }

            } else {
                spinner.println(format!("Couldn't find {anime_name}"));
            }
        };

        sleep(Duration::from_secs(1)).await;
    }

    spinner.finish_and_clear();

    println!("All Data Has been saved.");

    println!("Downloading Images...");
    // download anime images
    for (name, anime) in &results {
        let url = &anime.images.webp.large_image_url;
        let image_path = PathBuf::from(&config.images).join(format!("{name}.webp"));

        if let Err(error) = download_image(url, &image_path).await {
            eprintln!("Hell man, {error}");
        } else if *verbose {
            println!("Downloaded {url}.");
        };
    }

    Ok(())
}

const MAX_RETRY_COUNT: u8 = 5;
const RETRY_DELAY: u64 = 500;

async fn download_image (url: &str, out_path: &PathBuf) -> Result<()> {
    let mut attempts = 0;
    loop {
        let res = get(url).await;

        match res {
            Ok(res) if res.status() == reqwest::StatusCode::OK => {
                let buf = res.bytes().await?;
                fs::write(out_path, buf).await?;
                return Ok(());
            },
            Ok(res) => { eprintln!("Bad Response {}", res.status()) },
            Err(error) => { eprintln!("Error {error}"); }
        };

        attempts += 1;

        if attempts == MAX_RETRY_COUNT {
            return Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionReset,
                "Max retries exceeded",
            ).into());
        };

        let delay = Duration::from_millis(RETRY_DELAY * 2u64.pow(attempts as u32 - 1));
        sleep(delay).await;
    };
}

#[derive(Debug, FromRow)]
struct AnimeTitle {
    title: String
}

async fn get_anime_name(token: &str, pool: &SqlitePool) -> Result<Option<String>, sqlx::Error> {
    let row: Option<AnimeTitle> = sqlx::query_as("SELECT title FROM animes WHERE localName = $1")
        .bind(token)
        .fetch_optional(pool)
        .await?;
    
    Ok(row.map(|e| e.title))
}

async fn add_or_update_anime ((local_name, anime): &(&String, Anime), pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let studio_name = if !anime.studios.is_empty() {
        Some(&anime.studios[0].name)
    } else {
        None
    };

    let (broadcast_day, broadcast_time) = (&anime.broadcast.day, &anime.broadcast.time);

    let episodes = anime.episodes.unwrap_or(1);

    let result = sqlx::query(r#"
    INSERT INTO animes (
        mal_id, localName, title, title_english, title_japanese,
        type, source, episodes, status, aired_from, aired_to,
        duration, rating, score, popularity, rank, background,
        season, year, broadcast_day, broadcast_time, studio
    )
    VALUES (
        ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
    )
    ON CONFLICT(mal_id) DO UPDATE SET
        localName = excluded.localName,
        title = excluded.title,
        title_english = excluded.title_english,
        title_japanese = excluded.title_japanese,
        type = excluded.type,
        source = excluded.source,
        episodes = excluded.episodes,
        status = excluded.status,
        aired_from = excluded.aired_from,
        aired_to = excluded.aired_to,
        duration = excluded.duration,
        rating = excluded.rating,
        score = excluded.score,
        popularity = excluded.popularity,
        rank = excluded.rank,
        background = excluded.background,
        season = excluded.season,
        year = excluded.year,
        broadcast_day = excluded.broadcast_day,
        broadcast_time = excluded.broadcast_time,
        studio = excluded.studio;
    "#)
        .bind(anime.mal_id)
        .bind(local_name)
        .bind(&anime.title)
        .bind(&anime.title_english)
        .bind(&anime.title_japanese)
        .bind(&anime.r#type)
        .bind(&anime.source)
        .bind(episodes)
        .bind(&anime.status)
        .bind(&anime.aired.from)
        .bind(&anime.aired.to)
        .bind(&anime.duration)
        .bind(&anime.rating)
        .bind(anime.score)
        .bind(anime.popularity)
        .bind(anime.rank)
        .bind(&anime.background)
        .bind(&anime.season)
        .bind(anime.year)
        .bind(broadcast_day)
        .bind(broadcast_time)
        .bind(studio_name)
        .execute(pool)
        .await?;

    Ok(())
}
