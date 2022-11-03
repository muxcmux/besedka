use actix_web::get;
use actix_web::middleware::{Compress, Logger};
use actix_web::{
    App, HttpResponse, HttpRequest, HttpServer,
    web, http::header::ContentType
};
use sqlx::sqlite::SqlitePool;
use super::cli::Server;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::{
    fs::File,
    io::BufReader
};

impl Server {
    pub fn ssl(&self) -> bool {
        self.ssl_key.is_some() && self.ssl_cert.is_some()
    }

    pub fn tls(&self) -> rustls::ServerConfig {
        let cert = self.ssl_cert.clone().expect("Missing certificate file");
        let key = self.ssl_key.clone().expect("Missing key file");

        // init server config builder with safe defaults
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth();

        // load TLS key/cert files
        let cert_file = &mut BufReader::new(File::open(cert).expect("Can't open certificate file"));
        let key_file = &mut BufReader::new(File::open(key).expect("Can't open key file"));

        // convert files to key/cert objects
        let cert_chain = certs(cert_file)
            .expect("Failed creating certificate chain")
            .into_iter()
            .map(Certificate)
            .collect();
        let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)
            .expect("Failed making keys")
            .into_iter()
            .map(PrivateKey)
            .collect();

        // exit if no keys could be parsed
        if keys.is_empty() {
            eprintln!("Could not locate PKCS 8 private keys.");
            std::process::exit(1);
        }

        config.with_single_cert(cert_chain, keys.remove(0)).unwrap()
    }
}

pub async fn run(config: Server, db: SqlitePool) {
    log::debug!("{:#?}", config);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .wrap(Logger::default())
            .wrap(Compress::default())
            .service(web::resource("/").to(index))
            .service(comments)
            // .service(web::resource("/sites").to(sites))
    });

    log::info!("Starting Besedka on {}", config.bind);

    // This will block the main thread
    if config.ssl() {
        let _ = server.bind_rustls(&config.bind, config.tls()).unwrap().run().await;
    } else {
        let _ = server.bind(&config.bind).unwrap().run().await;
    }
}

async fn index(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().content_type(ContentType::html()).body(
        "<!DOCTYPE html><html><body>\
            <p>Hello besedka!</p>\
        </body></html>",
    )
}

#[get("/comments/{page:.*}")]
async fn comments(db: web::Data<SqlitePool>, path: web::Path<String>) -> web::Json<Vec<crate::db::comments::Comment>> {
    let page = path.into_inner();
    let comments = crate::db::comments::comments(&db, page).await.unwrap();
    web::Json(comments)
}
