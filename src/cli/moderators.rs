use sqlx::SqlitePool;

use crate::db::users::{User, all, insert_moderator};

pub async fn list(db: &SqlitePool, site: String) {
    match all(db, &site).await {
        Err(e) => println!("{}", e),
        Ok(moderators) => {
            println!("{} moderators: Found {}", site, moderators.len());
            for moderator in moderators {
                print_moderator(moderator);
            }
        }
    }
}

pub async fn create(db: &SqlitePool, moderator: super::ModeratorsAddCommand, site: String) {
    match insert_moderator(db, moderator, &site).await {
        Err(e) => println!("{}", e),
        Ok(result) => {
            println!("Success!");
            match result {
                None => println!("Moderator not found"),
                Some(m) => print_moderator(m)
            }
        }
    }
}

fn print_moderator(moderator: User) {
    println!(r#"
{}
{}
id:     {}
name:   {}
avatar: {}
"#,
        moderator.username,
        "-".repeat(moderator.username.len()),
        moderator.id,
        moderator.name,
        moderator.avatar.unwrap_or(String::from("-"))
    )
}

