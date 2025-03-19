// backend/src/main.rs
pub use self::error::{Error, Result}; // export types

mod db;
mod error;
mod models;
mod web;

use axum::middleware;

use axum::response::Response;
// Axum is a web framework for Rust (It is to rust what express is to node.js)
use axum::{routing::get_service, Extension, Router};
use dotenv::dotenv;
use std::net::SocketAddr; // Allows us to bind the backend to a specific port
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer; // Provides support for GET/POST/PUT/DELETE/PATCH/OPTIONS // Load .Env
use tower_http::services::ServeDir;
use http::header::{HeaderName, HeaderValue};
use http::Method;

use crate::db::pool::create_pool; // Import the connection pool

#[tokio::main] // Indicates that the main function is an async function using tokiopub mod web;
async fn main() {
    // Load environment variables from .env file
    dotenv().ok();

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
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("authorization"),
            HeaderName::from_static("x-requested-with"),
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

    let cookie_layer = CookieManagerLayer::new();

    let app = Router::new()
        .merge(web::routes::db_controller::db_routes())
        .nest("/api", user_api_routes) // Merge routes from user_controller,  any routes defined in user controller are now part of the Axum application.
        .nest("/api/document", doc_api_routes) // Merge routes from document_controller
        .layer(Extension(pool)) // Make the pool available to all handlers,Attachs the PgPool as an Axum Extension
        .layer(middleware::map_response(main_response_mapper))
        .layer(cookie_layer)
        .layer(cors) // Add the CORS layer
        .fallback_service(routes_static()); // Fallback route if route cannot be found above

    /*
    / Bind the router to a specific port
    / We use the SocketAddr struct to bind the router to the port
    / We use the 0.0.0.0 address to bind the router to localhost
    / We will bind to port 3001 for now
    / We print a message to the console to indicate that the server is starting
    */
    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    println!("Server starting on http://localhost:3001");

    /*
    / Serve the router ie Start the server
    / We will start the server with the configured router and address
    */
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Fallback Route If One Cannot Be Resolved
fn routes_static() -> Router {
    println!("->> {:<12} - routes_static", "FALLBACK HIT");
    Router::new().nest_service("/", get_service(ServeDir::new("./")))
}

async fn main_response_mapper(response: Response) -> Response {
    println!("->> {:<12} - main_response_mapper", "RES_MAPPER");

    println!();

    response
}
