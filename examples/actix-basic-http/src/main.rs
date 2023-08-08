use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use opentelemetry::global;
use std::io;
use tembo_telemetry::{TelemetryConfig, TelemetryInit};
use tracing::*;
use tracing_actix_web::TracingLogger;

const TRACER_NAME: &str = "tembo.io/trace";

#[get("/")]
#[instrument]
async fn hello(tc: web::Data<TelemetryConfig>) -> impl Responder {
    let trace_id = tc.get_trace_id();
    Span::current().record("trace_id", &field::display(&trace_id));
    info!("Received request for /hello");
    HttpResponse::Ok().json("Hello World!")
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    // Setup Telemetry and Logging
    let telemetry_config = if let Ok(otlp_endpoint) = std::env::var("OPENTELEMETRY_ENDPOINT_URL") {
        let tc = TelemetryConfig {
            app_name: std::env::var("CARGO_BIN_NAME").unwrap_or_else(|_| "basic".to_string()),
            env: std::env::var("ENV").unwrap_or_else(|_| "development".to_string()),
            endpoint_url: Some(otlp_endpoint),
            tracer_id: Some(TRACER_NAME.to_string()),
        };
        println!("{:?}", tc);
        let _telemetry = TelemetryInit::init(&tc).await;
        tc
    } else {
        let tc = TelemetryConfig {
            app_name: std::env::var("ENV").unwrap_or_else(|_| "development".to_string()),
            env: std::env::var("ENV").unwrap_or_else(|_| "development".to_string()),
            endpoint_url: None,
            tracer_id: Some(TRACER_NAME.to_string()),
        };
        let _telemetry = TelemetryInit::init(&tc).await;
        tc
    };

    let server_bind_address = "0.0.0.0:3001".to_string();
    let server = HttpServer::new({
        let telemerty_config = web::Data::new(telemetry_config);
        move || {
            App::new()
                .app_data(telemerty_config.clone())
                .service(hello)
                .wrap(TracingLogger::default())
        }
    })
    .bind(server_bind_address.clone())?
    .shutdown_timeout(5)
    .run();

    info!(
        "Starting HTTP server at https://{}/",
        server_bind_address.clone()
    );
    server.await?;

    global::shutdown_tracer_provider();

    Ok(())
}
