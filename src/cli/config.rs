use sqlx::{SqlitePool, query_as};
use crate::cli::ConfigSetCommand;
use crate::db::configs::{find, Config};

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

async fn all(db: &SqlitePool) -> anyhow::Result<Vec<Config>> {
    let configs = query_as!(Config, "SELECT * FROM configs")
        .fetch_all(db).await?;
    Ok(configs)
}

async fn upsert(db: &SqlitePool, site: ConfigSetCommand) -> anyhow::Result<Config>{
    let mut query = String::from("INSERT INTO configs (site");
    let mut values = String::from("VALUES (?");
    let mut conflict = String::from("ON CONFLICT(site) DO UPDATE SET site=excluded.site RETURNING *");

    append_to_upsert(site.anonymous_comments, "anonymous_comments", &mut query, &mut values, &mut conflict);
    append_to_upsert(site.moderated, "moderated", &mut query, &mut values, &mut conflict);
    append_to_upsert(site.comments_per_page, "comments_per_page", &mut query, &mut values, &mut conflict);
    append_to_upsert(site.replies_per_comment, "replies_per_comment", &mut query, &mut values, &mut conflict);
    append_to_upsert(site.minutes_to_edit, "minutes_to_edit", &mut query, &mut values, &mut conflict);
    append_to_upsert(site.theme.clone(), "theme", &mut query, &mut values, &mut conflict);

    query.push_str(") ");
    values.push_str(") ");
    query.push_str(&values);
    query.push_str(&conflict);

    let mut config = query_as::<_, Config>(&query);

    config = config.bind(site.site.clone());
    if let Some(a) = site.anonymous_comments { config = config.bind(a) }
    if let Some(a) = site.moderated { config = config.bind(a) }
    if let Some(a) = site.comments_per_page { config = config.bind(a) }
    if let Some(a) = site.replies_per_comment { config = config.bind(a) }
    if let Some(a) = site.minutes_to_edit { config = config.bind(a) }
    if let Some(a) = site.theme { config = config.bind(a) }
    config = config.bind(site.site);

    Ok(config.fetch_one(db).await?)
}

fn append_to_upsert<T>(value: Option<T>, attribute: &str, query: &mut String, values: &mut String, conflict: &mut String) {
    if let Some(_) = value {
        query.push_str(&format!(", {}", attribute));
        values.push_str(", ?");
        conflict.push_str(&format!(", {}=excluded.{}", attribute, attribute));
    }
}

async fn regenerate_secret(db: &SqlitePool, site: &str) -> anyhow::Result<Config> {
    let config = query_as!(Config, r#"
        UPDATE configs SET secret = (randomblob(32)) WHERE(site = ?);
        SELECT * FROM configs WHERE(site = ?);
    "#, site, site)
        .fetch_one(db)
        .await?;
    Ok(config)
}
