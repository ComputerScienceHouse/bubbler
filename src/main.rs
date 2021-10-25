extern crate actix_web;

use actix_web::{web, App, HttpServer};
use std::sync::Mutex;

pub mod routes;
use routes::config::{AppData, ConfigData};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    ConfigData::new().initialize_slots().unwrap();

    HttpServer::new(|| {
        App::new()
            .app_data(
                web::Data::new(Mutex::new(AppData {
                    config: ConfigData::new()
                }))
            )
            .service(routes::drop)
            .service(routes::health)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
