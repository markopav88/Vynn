// backend/src/main.rs
pub use self::error::{Error, Result}; // export types

mod db;
mod error;
mod models;
mod web;
mod rag;
mod log;

use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::{routing::get_service, Extension, Json, Router};
use dotenv::dotenv;
use http::header::HeaderValue;
use http::{Method, Uri, Request};
use log::log_request;
use uuid::Uuid;
use std::net::SocketAddr; // Allows us to bind the backend to a specific port
use std::env; // Import env module
use std::str::FromStr; // Import FromStr trait for SocketAddr parsing
use tower_cookies::{CookieManagerLayer, Cookies};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use serde_json::json; // Import the json! macro
use axum::middleware::Next;
use axum::body::Body;

use crate::db::pool::create_pool; // Import the connection pool

#[tokio::main] // Indicates that the main function is an async function using tokiopub mod web;
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    // Read environment variables
    let api_base_url = env::var("API_BASE_URL").expect("API_BASE_URL must be set");
    let front_end_url = env::var("FRONTEND_URL").expect("FRONTEND_URL must be set");
    let bind_address = env::var("BIND_ADDRESS").expect("BIND_ADDRESS must be set");

    /*
    / Creating the Pool using SQLx
    / Creates the pool before building router
    / connects to db using the DATABASE_URL from environment and returns a PgPool
    */
    let pool = create_pool().await;

    /*
    / Configure CORS
    / CORS is needed when a frontend (running on one domain or port)
    / wants to send HTTP requests to a backend running on another domain or port.
    / This is needed for the frontend to send requests to the backend.
    / We allow all origins, methods, and headers currently, but this should be changed later for security.
    */
    let cors = CorsLayer::new()
        .allow_origin(front_end_url.parse::<HeaderValue>().expect("Invalid api_base_url format"))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([
            http::header::CONTENT_TYPE,
            http::header::ACCEPT,
            http::header::AUTHORIZATION,
            http::header::HeaderName::from_static("x-requested-with"),
        ])
        .allow_credentials(true);

    /*
    / Initialize our router
    / We have one route for now: /api/hello
    / This route is used to test the backend
    / When the route is hit, the hello function is called
    */
    let user_api_routes = web::routes::user_controller::user_routes();
    let doc_api_routes = web::routes::doc_controller::doc_routes();
    let db_api_routes = web::routes::db_controller::db_routes();
    let project_api_routes = web::routes::proj_controller::project_routes();
    let key_api_routes = web::routes::key_controller::key_routes();
    let writing_assistant_routes = web::routes::ai_controller::writing_assistant_routes();
    let pref_api_routes = web::routes::pref_controller::pref_routes();

    let cookie_layer = CookieManagerLayer::new();

    let app = Router::new()
        .nest("/api/db", db_api_routes) // Merge routes from db_controller
        .nest("/api/users", user_api_routes) // Merge routes from user_controller
        .nest("/api/document", doc_api_routes) // Merge routes from document_controller
        .nest("/api/project", project_api_routes)
        .nest("/api/command", key_api_routes)
        .nest("/api/writing-assistant", writing_assistant_routes)
        .nest("/api/preference", pref_api_routes)
        .layer(Extension(pool.clone())) // Make the pool available to all handlers,Attachs the PgPool as an Axum Extension
        .layer(middleware::from_fn(mw_log_requests))
        .layer(cookie_layer)
        .layer(cors) // Add the CORS layer
        .fallback_service(routes_static()); // Fallback route if route cannot be found above

    /*
    / Bind the router to a specific port
    / We use the SocketAddr struct to bind the router to the port
    / We use the 0.0.0.0 address to bind the router to localhost
    / We will bind to port 3001 for now
    */
    let addr = SocketAddr::from_str(&bind_address).expect("Invalid BIND_ADDRESS format");
    println!("Server starting on {}", api_base_url); // Log the configured bind address

    /*
    / Serve the router ie Start the server
    / We will start the server with the configured router and address
    */
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

// Fallback Route If One Cannot Be Resolved
fn routes_static() -> Router {
    println!("->> {:<12} - routes_static", "FALLBACK HIT");
    Router::new().nest_service("/", get_service(ServeDir::new("./")))
}

// Custom logging middleware
async fn mw_log_requests(
    cookies: Cookies, // Extractor for cookies
    req_method: Method, // Extractor for method
    uri: Uri,         // Extractor for URI
    request: Request<Body>, // The request itself
    next: Next<Body>         // The next service in the chain
) -> Result<Response> { // Changed return type to Result<Response>
    println!("->> {:<12} - mw_log_requests", "MIDDLEWARE");
    let uuid = Uuid::new_v4();

    // Execute the rest of the stack to get the response
    let response = next.run(request).await;

    // --- Log Response ---
    // Get eventual response error from extensions
    let service_error = response.extensions().get::<Error>();
    let client_status_error = service_error.map(|e| e.client_status_and_error());

    // If client error, map a new response (This part replaces main_response_mapper's error handling)
    let error_response = 
        client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body = json!({
                "error": {
                    "type": client_error.as_ref(),
                    "req_uuid": uuid.to_string(),
                }
            });

            println!("  ->> client_error_body: {client_error_body}");

            // Build new response for client
            (*status_code, Json(client_error_body)).into_response()
        });

    // Build and log server log line
    let client_error = client_status_error.unzip().1;
    // Use cloned cookies for logging as original `cookies` might be consumed by the extractor.
    // We pass the necessary extracted info directly.
    let log_result = log_request(uuid, req_method, uri, service_error, client_error, &cookies).await;

    // Handle potential logging errors if necessary (e.g., log to stderr)
    if let Err(e) = log_result {
        eprintln!("->> {:<12} - FAILED TO LOG REQUEST: {:?}", "ERROR", e);
    }

    println!();
    // Return the mapped error response if it exists, otherwise the original response
    Ok(error_response.unwrap_or(response))
}
