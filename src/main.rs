use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use failure::format_err;
use serde::{Deserialize, Serialize};
use std::process::Command;

type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug, Serialize, Deserialize)]
struct ReqInitialize {
    payment_service_url: String,
    shipment_service_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResInitialize {
    campaign: i32,
    language: String,
}

fn post_initialize(ri: web::Json<ReqInitialize>) -> Result<web::Json<ResInitialize>> {
    let status = Command::new("../sql/init.sh").status()?;
    if !status.success() {
        return Err(format_err!("exec init.sh error"));
    }
    Ok(web::Json(ResInitialize {
        campaign: 0,
        language: "Rust".to_string(),
    }))
}

fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(web::resource("/initialize").route(web::post().to(post_initialize)))
    })
    .bind("127.0.0.1:8000")?
    .run()
}
