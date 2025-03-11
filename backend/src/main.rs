// backend/src/main.rs
pub use self::error::{Error, Result}; // export types

mod db;
mod error;
mod models;
mod web;

use axum::middleware;

// Axum is a web framework for Rust (It is to rust what express is to node.js)
use axum::{routing::get_service, Extension, Router};
use dotenv::dotenv;
use std::net::SocketAddr; // Allows us to bind the backend to a specific port
use tower_http::cors::{Any, CorsLayer}; // Provides support for GET/POST/PUT/DELETE/PATCH/OPTIONS // Load .Env
use tower_http::services::ServeDir;

use crate::db::pool::create_pool; // Import the connection pool
use crate::web::routes::db_controller::db_routes;
use crate::web::routes::user_controller::user_routes; // Import user routes from user controller // Import the migrate_db function

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
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    /*
    / Initialize our router
    / We have one route for now: /api/hello
    / This route is used to test the backend
    / When the route is hit, the hello function is called
    */
    let app = Router::new()
        .merge(db_routes())
        .merge(user_routes()) // Merge routes from user_controller,  any routes defined in user controller are now part of the Axum application.
        .layer(Extension(pool)) // Make the pool available to all handlers,Attachs the PgPool as an Axum Extension
        .layer(cors)
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
    / We will star the server with the configured router and address
    */
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Fallback Route If One Cannot Be Resolved
fn routes_static() -> Router {
    Router::new().nest_service("/", get_service(ServeDir::new("./")))
}
