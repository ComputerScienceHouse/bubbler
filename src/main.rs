extern crate actix_web;

use std::sync::{Arc, Mutex};

use actix_web::{web, App, HttpServer};

pub mod routes;
use routes::config::{AppData, ConfigData};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config_data = ConfigData::new();
    config_data.initialize_slots().unwrap();

    let app_data = web::Data::new(AppData {
        config: config_data,
        drop_lock: Arc::new(Mutex::new(())),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(routes::drop)
            .service(routes::health)
            .service(routes::get_slots)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
