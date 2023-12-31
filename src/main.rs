mod context;
mod settings;
mod translation_api;
mod translator;

use std::net::Ipv6Addr;

use actix_web::{get, web::Data, App, HttpResponse, HttpServer, Responder};
use context::prepare_app_context;
use settings::SETTINGS;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::EnvFilter;

use crate::open_api::OPEN_API;

mod open_api;

#[get("/openapi.json")]
async fn openapi_json() -> impl Responder {
    HttpResponse::Ok().json(&*OPEN_API)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Force settings evaluation in main
    let _ = SETTINGS.path;

    let app_context = Data::new(prepare_app_context());

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .configure(translation_api::configure(app_context.clone()))
            .service(openapi_json)
    })
    // .bind((Ipv4Addr::UNSPECIFIED, 8080))?
    .bind((Ipv6Addr::UNSPECIFIED, 8080))?
    .run()
    .await
}
