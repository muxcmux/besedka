use crate::db::configs::{find, insert, update, Config, all, self};
use sqlx::SqlitePool;

pub async fn print(db: &SqlitePool, site: &str) {
    match find(db, &site).await {
        Err(e) => println!("{}", e),
        Ok(cfg) => match cfg {
            None => println!("No config found for {}. Try adding it with:\n$ besedka config set {}", site, site),
            Some(c) => print_config(&c),
        },
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

pub async fn delete(db: &SqlitePool, site: &str) {
    match find(db, &site).await {
        Err(e) => println!("{}", e),
        Ok(cfg) => match cfg {
            None => println!("No config found for {}", site),
            Some(_) => {
                match configs::delete(db, site).await {
                    Err(e) => println!("{}", e),
                    Ok(_) => println!("Deleted config for {}", site),
                }
            }
        },
    }
}

pub async fn create_or_update(db: &SqlitePool, args: super::ConfigSetCommandArgs) {
    let config = match find(db, &args.site).await.unwrap() {
        None => insert(db, args).await.unwrap(),
        Some(existing) => update(db, existing, args).await.unwrap()
    };
    println!("Success!");
    print_config(&config);
}

fn print_config(cfg: &Config) {
    println!(
        r#"
{}
{}
secret:              {}
private:             {}
anonymous:           {}
moderated:           {}
comments_per_page:   {}
replies_per_comment: {}
minutes_to_edit:     {}
theme:               {}
"#,
        cfg.site,
        "-".repeat(cfg.site.len()),
        cfg.secret(),
        cfg.private,
        cfg.anonymous,
        cfg.moderated,
        cfg.comments_per_page,
        cfg.replies_per_comment,
        cfg.minutes_to_edit,
        cfg.theme
    );
}
