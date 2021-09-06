extern crate actix_web;

use actix_web::{web, App, HttpServer};
use std::sync::Mutex;

pub mod routes;
use routes::config::AppData;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(
                web::Data::new(Mutex::new(AppData {
                    config: routes::config::init() 
                }))
            )
            .service(routes::drop)
            .service(routes::health)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
