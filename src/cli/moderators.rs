use sqlx::SqlitePool;

use crate::db::moderators::{Moderator, all, insert_moderator, find_by_name, delete};

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

pub async fn remove(db: &SqlitePool, name: &str) {
    match find_by_name(db, name).await {
        Err(_) => {
            println!("Moderator {} not found.", name)
        },
        Ok(_) => {
            delete(db, name).await.unwrap();
            println!("Deleted moderator {}", name);
        }
    }
}

fn print_moderator(moderator: Moderator) {
    println!(r#"
{}
{}
op: {}
avatar: {}
"#,
        moderator.name,
        "-".repeat(moderator.name.len()),
        moderator.op,
        match moderator.avatar_id { Some(_) => "Yes", None => "No" }
    )
}

