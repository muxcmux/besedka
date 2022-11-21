use crate::db::sites::{find, insert, Site, all, self};
use sqlx::SqlitePool;

pub async fn print(db: &SqlitePool, site: &str) {
    match find(db, &site).await {
        Err(e) => println!("{}", e),
        Ok(cfg) => match cfg {
            None => println!("Site {} not found. Try adding it first:\n$ besedka site add {}", &site, &site),
            Some(c) => print_site(&c),
        },
    }
}

pub async fn list(db: &SqlitePool) {
    match all(db).await {
        Err(e) => println!("{}", e),
        Ok(configs) => {
            println!("Found {} site(s)", configs.len());
            for cfg in configs {
                print_site(&cfg);
            }
        }
    }
}

pub async fn delete(db: &SqlitePool, site: &str) {
    match find(db, &site).await {
        Err(e) => println!("{}", e),
        Ok(cfg) => match cfg {
            None => println!("Site {} not found.", site),
            Some(_) => {
                match sites::delete(db, site).await {
                    Err(e) => println!("{}", e),
                    Ok(_) => println!("Deleted config for site {}", site),
                }
            }
        },
    }
}

pub async fn create(db: &SqlitePool, args: super::SitesCommandArgs) {
    let site = args.site.clone();
    match insert(db, args).await {
        Err(err) => match err {
            sqlx::Error::Database(e) if e.message().contains("UNIQUE") => {
                println!("Site {} already exists. To update the config, use:\n$ besedka site update {}", &site, &site)
            },
            _ => println!("Site {} not found. Try adding it first:\n$ besedka site add {}", &site, &site),
        },
        Ok(site) => {
            println!("Success!");
            print_site(&site);
        }
    }
}

pub async fn update(db: &SqlitePool, args: super::SitesCommandArgs) {
    match find(db, &args.site).await.unwrap() {
        None => println!("Site {} not found. Try adding it first:\n$ besedka site add {}", &args.site, &args.site),
        Some(existing) => {
            let site = sites::update(db, existing, args).await.unwrap();
            println!("Success!");
            print_site(&site);
        }
    }
}

fn print_site(cfg: &Site) {
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
