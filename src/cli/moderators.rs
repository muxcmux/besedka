use sqlx::SqlitePool;

use crate::db::moderators::{self, Moderator, all, insert_moderator, find_by_name, delete};

use super::{ModeratorsAddCommandArgs, ModeratorsUpdateCommandArgs};

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

pub async fn create(db: &SqlitePool, moderator: ModeratorsAddCommandArgs) {
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

pub async fn update(db: &SqlitePool, args: ModeratorsUpdateCommandArgs) {
    match find_by_name(db, &args.name).await {
        Err(_) => {
            println!("Moderator {} not found.", &args.name)
        },
        Ok(_) => {
            let updated = moderators::update(&db, &args.name, args.op, args.avatar, args.password).await;
            println!("Success!");
            print_moderator(updated.unwrap());
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
        match moderator.avatar { Some(a) => a, None => String::from("false") }
    )
}

