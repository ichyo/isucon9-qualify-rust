use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use failure::format_err;
use r2d2::Pool;
use r2d2_mysql::MysqlConnectionManager;
use serde::{Deserialize, Serialize};
use std::{env, process};

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

fn post_initialize(
    ri: web::Json<ReqInitialize>,
    db: web::Data<Pool<MysqlConnectionManager>>,
) -> Result<web::Json<ResInitialize>> {
    let status = process::Command::new("../sql/init.sh").status()?;
    if !status.success() {
        return Err(format_err!("exec init.sh error"));
    }
    let mut conn = db.get()?;

    conn.prep_exec(
        r#"
    INSERT INTO `configs` (`name`, `val`) VALUES (?, ?)
    ON DUPLICATE KEY UPDATE `val` = VALUES(`val`)
    "#,
        ("payment_service_url", &ri.payment_service_url),
    )?;

    conn.prep_exec(
        r#"
    INSERT INTO `configs` (`name`, `val`) VALUES (?, ?)
    ON DUPLICATE KEY UPDATE `val` = VALUES(`val`)
    "#,
        ("shipment_service_url", &ri.shipment_service_url),
    )?;

    Ok(web::Json(ResInitialize {
        campaign: 0,
        language: "Rust".to_string(),
    }))
}

fn main() -> Result<()> {
    env_logger::init();

    let host = env::var("MYSQL_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("MYSQL_PORT")
        .unwrap_or_else(|_| "3306".to_string())
        .parse::<u16>()?;
    let user = env::var("MYSQL_USER").unwrap_or_else(|_| "isucari".to_string());
    let db_name = env::var("MYSQL_DBNAME").unwrap_or_else(|_| "isucari".to_string());
    let password = env::var("MYSQL_PASS").unwrap_or_else(|_| "isucari".to_string());

    let mut opts = mysql::OptsBuilder::new();
    opts.ip_or_hostname(Some(host))
        .tcp_port(port)
        .user(Some(user))
        .pass(Some(password))
        .db_name(Some(db_name));

    let manager = MysqlConnectionManager::new(opts);
    let pool = Pool::new(manager)?;

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(Logger::default())
            .service(web::resource("/initialize").route(web::post().to(post_initialize)))
            .service(actix_files::Files::new("/", "../public"))
    })
    .bind("127.0.0.1:8000")?
    .run()?;

    Ok(())
}
