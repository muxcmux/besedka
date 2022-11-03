use sqlx::SqlitePool;

use crate::db::configs::{find, all, Config, upsert, regenerate_secret};

pub async fn print(db: &SqlitePool, site: String) {
    match find(db, &site).await {
        Err(e) => println!("{}", e),
        Ok(cfg) => match cfg {
            None => println!("Couldn't find {} configuration", site),
            Some(c) => print_config(&c)
        }
    }
}

pub async fn list(db: &SqlitePool) {
    match all(db).await {
        Err(e) => println!("{}", e),
        Ok(configs) => {
            println!("Found {} configuration(s)", configs.len());
            for cfg in configs {
                print_config(&cfg);
            }
        }
    }
}

pub async fn create_or_update(db: &SqlitePool, config: super::ConfigSetCommand) {
    match upsert(db, config).await {
        Err(e) => println!("{}", e),
        Ok(result) => {
            println!("Success!");
            print_config(&result);
        }
    }
}

pub async fn reset_secret(db: &SqlitePool, site: String) {
    match find(db, &site).await.unwrap() {
        None => println!("Couldn't find {} configuration", site),
        Some(_) => {
            println!("Success!");
            print_config(&regenerate_secret(db, &site).await.unwrap())
        }
    }
}

fn print_config(cfg: &Config) {
    println!(r#"
{}
{}
secret:              {}
anonymous_comments:  {}
moderated:           {}
comments_per_page:   {}
replies_per_comment: {}
minutes_to_edit:     {}
theme:               {}
"#,
        cfg.site,
        "-".repeat(cfg.site.len()),
        cfg.secret(),
        cfg.anonymous_comments,
        cfg.moderated,
        cfg.comments_per_page,
        cfg.replies_per_comment,
        cfg.minutes_to_edit,
        cfg.theme
    );
}
