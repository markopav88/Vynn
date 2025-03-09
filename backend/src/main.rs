// backend/src/main.rs
mod db;
mod controllers;
mod models;

use hyper::Server;

use axum::{
    routing::get,
    Router,
    Json,
    Extension
}; // Axum is a web framework for Rust (It is to rust what express is to node.js)
use std::net::SocketAddr; // Allows us to bind the backend to a specific port 
use tower_http::cors::{CorsLayer, Any}; // Provides support for GET/POST/PUT/DELETE/PATCH/OPTIONS
use sqlx::Row;
// Import the connection pool 
use crate::db::pool::create_pool;
// Import user-related routes from controller 
use crate::controllers::user_controller::user_routes;

#[tokio::main] // Indicates that the main function is an async function using tokio
async fn main() {
        dotenvy::dotenv().ok();
    //Creating the Pool using SQLx
    //Creates the pool before building router
    let pool = create_pool().await;
    //connects to db using the DATABASE_URL from environment and returns a PgPool
   
   
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
        .route("/api/hello", get(hello))
        .route("/api/test-db", get(test_db))
        .merge(user_routes())  // Merge routes from user_controller,  any routes defined in user controller are now part of the Axum application.
        .layer(Extension(pool)) // Make the pool available to all handlers,Attachs the PgPool as an Axum Extension
        .layer(cors);

    /*
    / Bind the router to a specific port
    / We use the SocketAddr struct to bind the router to the port
    / We use the 0.0.0.0 address to bind the router to localhost
    / We will bind to port 3000 for now 
    / We print a message to the console to indicate that the server is starting
    */
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server starting on http://localhost:3000");
    
    /*
    / Serve the router ie Start the server
    / We will star the server with the configured router and address
    */
    // Start the Axum server using the configured router and address
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    
}

/*
/ Define the hello function
/ This function is called when the /api/hello route is hit
/ It returns a JSON object with a message
/ Should most likely be moved to a controller file later but its here for now
*/
async fn hello() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "message": "Hello from Rust!"
    }))
}
async fn test_db(
    Extension(pool): Extension<sqlx::PgPool>,
) -> Json<serde_json::Value> {
    // Run a simple query to test the database connection.
    let row = sqlx::query("SELECT 1 as number")
        .fetch_one(&pool)
        .await
        .expect("Query failed");

    let number: i32 = row.try_get("number")
        .expect("Failed to get column 'number'");

    Json(serde_json::json!({ "result": number }))
}
    