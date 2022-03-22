extern crate serde_json;

use actix_web::{get, post, web, HttpResponse, Responder};
use actix_web::http::StatusCode;
use serde::{Deserialize, Serialize};

pub mod machine;
pub mod config;
use config::AppData;
use machine::DropError;

#[derive(Serialize, Deserialize)]
struct HealthReport {
    slots: Vec<String>,
    temp: f32,
}
#[derive(Serialize)]
struct SlotReport {
    slots: Vec<machine::SlotStatus>,
    temp: f32,
}

#[derive(Serialize, Deserialize)]
struct DropRequest {
    slot: usize,
}

#[derive(Serialize)]
struct DropResponse {
    message: String,
}

#[derive(Serialize)]
#[allow(non_snake_case)]
struct DropErrorRes {
    error: String,
    errorCode: u16,
}

#[post("/drop")]
async fn drop(data: web::Data<AppData>, req_body: web::Json<DropRequest>) -> impl Responder {
    match machine::drop(data.config.clone(), req_body.slot) {
        Ok(_) => HttpResponse::Ok().json(DropResponse {
            message: "Dropped drink from slot ".to_string() + &req_body.slot.to_string()
        }),
        Err(DropError::BadSlot) => {
            HttpResponse::Ok().status(StatusCode::BAD_REQUEST).json(DropErrorRes {
                error: "Invalid slot ID provided".to_string(),
                errorCode: 400
            })
        },
        Err(DropError::MotorFailed) => {
            HttpResponse::Ok().status(StatusCode::INTERNAL_SERVER_ERROR).json(DropErrorRes {
                error: "Motor failed to actuate".to_string(),
                errorCode: 500
            })
        }
    }
}

#[get("/health")]
async fn health(data: web::Data<AppData>) -> impl Responder {
    let slots = machine::get_slots_old(data.config.clone());
    let temperature = machine::get_temperature(data.config.clone());

    let temperature = temperature * (9.0/5.0) + 32.0;

    HttpResponse::Ok().json(HealthReport {
        slots: slots.to_vec(),
        temp: temperature,
    })
}

#[get("/slots")]
async fn get_slots(data: web::Data<AppData>) -> impl Responder {
    let slots = machine::get_slots(data.config.clone());
    let temp = machine::get_temperature(data.config.clone());

    HttpResponse::Ok().json(SlotReport {
        slots,
        temp,
    })
}
