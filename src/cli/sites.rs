use crate::db::sites::{find, insert, Site, all, self};
use sqlx::SqlitePool;

pub async fn print(db: &SqlitePool, site: &str) {
    match find(db, &site).await {
        Err(_) => println!("Site {} not found. Try adding it first:\n$ besedka site add {}", &site, &site),
        Ok(cfg) => print_site(&cfg)
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
        Err(_) => println!("Site {} not found.", site),
        Ok(_) => match sites::delete(db, site).await {
            Err(e) => println!("{}", e),
            Ok(_) => println!("Deleted config for site {}", site)
        },
    }
}

pub async fn create(db: &SqlitePool, args: super::SitesCommandArgs) {
    let site = &args.site;
    match insert(db, &args).await {
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
    match find(db, &args.site).await {
        Err(_) => println!("Site {} not found. Try adding it first:\n$ besedka site add {}", &args.site, &args.site),
        Ok(existing) => match sites::update(db, existing, args).await {
            Err(e) => println!("{}", e),
            Ok(s) => {
                println!("Success!");
                print_site(&s);
            }
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
"#,
        cfg.site,
        "-".repeat(cfg.site.len()),
        cfg.secret(),
        cfg.private,
        cfg.anonymous,
        cfg.moderated,
    );
}
