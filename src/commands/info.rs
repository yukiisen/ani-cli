use ani_core::models::Anime;
use ani_core::database;
use database::queries::anime::*;

use sqlx::SqlitePool;

pub async fn display_anime_info (mal_id: &u32, detailed: bool, pool: SqlitePool) {
    let anime = get_anime_by_id(*mal_id, &pool).await;

    match anime {
        Ok(Some(anime)) => {
            disply_data(anime, detailed);
        },
        Ok(None) => {},
        Err(err) => { eprintln!("{err}"); }
    }
}


fn disply_data (anime: Anime, detailed: bool) {
    let separator = "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━";
    println!("{separator}");
    println!("  Name:          {} ({})", anime.title, anime.r#type);
    println!("  English Name:  {}", anime.title_english.as_deref().unwrap_or_default());
    println!("  Japanese Name: {}", anime.title_japanese.as_deref().unwrap_or_default());
    println!("{separator}");
    println!("  ID:            {}", anime.id);
    println!("  MALID:         {}", anime.mal_id);
    println!("  Aired:         From {} to {}", anime.get_aired_from().date_naive(),
                                  anime.get_aired_to().date_naive());
    println!("  Episodes:      {} ({})", anime.episodes, anime.status);
    println!("  Rating:        {}", anime.rating);
    println!("  Score:         {} (#{})", anime.score.unwrap_or_default(), 
                                     anime.rank.unwrap_or(0));
    println!("  Studio:        {}", anime.studio.as_deref().unwrap_or_default());

    if detailed {
        println!("{separator}");
        println!("  Popularity:    {}", anime.popularity);
        if !anime.background.is_empty() { 
            let mut index = 0;
            while index < anime.background.len() {
                let slice = &anime.background[index..std::cmp::min(index+50, anime.background.len())];
                index += 50;
                let bg_string = if index == 50 { "Background:" } else { "           " };
                println!("  {bg_string}    {}", slice); 
            }
        };
        println!("  Season:        {1} {0}", anime.season.as_deref().unwrap_or_default(), anime.year.unwrap_or(0));
        println!("  Broadcast:     {} at {}", anime.broadcast_day.as_deref().unwrap_or_default(),
                                          anime.broadcast_time.as_deref().unwrap_or_default());
    }

    println!("{separator}");
}
