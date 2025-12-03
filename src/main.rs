mod config;
mod db;
mod handlers;
mod models;
mod services;
mod utils;

use actix_files as fs;
use actix_web::{middleware, web, App, HttpServer};
use config::Config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init();

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");
    let server_address = config.server_address();

    log::info!("Starting Cross Bow server...");
    log::info!("Configuration loaded successfully");

    // Create database pool
    let pool = db::create_pool(&config.database_url, config.max_connections)
        .await
        .expect("Failed to create database pool");

    log::info!("Database connection established");
    log::info!("Running database migrations...");

    log::info!("Server starting on http://{server_address}");
    log::info!("🌐 Click here to open: http://localhost:{}", config.port);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            // Add logger middleware
            .wrap(middleware::Logger::default())
            // Add shared state
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            // API routes
            .route("/webhooks/github", web::post().to(handlers::github_webhook))
            .route(
                "/webhook/{source}",
                web::post().to(handlers::generic_webhook),
            )
            // Web interface routes
            .route("/", web::get().to(handlers::dashboard))
            .route("/repositories", web::get().to(handlers::list_repositories))
            .route(
                "/repositories/{id}",
                web::get().to(handlers::repository_detail),
            )
            .route("/events", web::get().to(handlers::list_events))
            // Static file serving
            .service(fs::Files::new("/assets", "./assets").show_files_listing())
    })
    .bind(&server_address)?
    .run()
    .await
}
