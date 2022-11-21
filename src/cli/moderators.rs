use sqlx::SqlitePool;

use crate::db::moderators::{Moderator, all, insert_moderator};

pub async fn list(db: &SqlitePool) {
    match all(db).await {
        Err(e) => println!("{}", e),
        Ok(moderators) => {
            println!("Moderators: Found {}", moderators.len());
            for moderator in moderators {
                print_moderator(moderator);
            }
        }
    }
}

pub async fn create(db: &SqlitePool, moderator: super::ModeratorsAddCommandArgs) {
    match insert_moderator(db, moderator).await {
        Err(error) => {
            match error {
                sqlx::Error::Database(e) if e.message().contains("UNIQUE") => {
                    println!("Moderator with that name already exists");
                },
                _ => println!("{}", error),
            }
        },
        Ok(result) => {
            println!("Success!");
            print_moderator(result);
        }
    }
}

fn print_moderator(moderator: Moderator) {
    println!(r#"
{}
{}
avatar: {}
"#,
        moderator.name,
        "-".repeat(moderator.name.len()),
        moderator.avatar.unwrap_or(String::from("-"))
    )
}

