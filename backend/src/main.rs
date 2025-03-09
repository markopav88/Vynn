// backend/src/main.rs
mod db;
mod controllers;
mod models;

// Axum is a web framework for Rust (It is to rust what express is to node.js)
use axum::{
    routing::get,
    Router,
    Extension
};
use std::net::SocketAddr; // Allows us to bind the backend to a specific port 
use std::env; // Allows us read arguments
use tower_http::cors::{CorsLayer, Any}; // Provides support for GET/POST/PUT/DELETE/PATCH/OPTIONS
use dotenv::dotenv; // Load .Env

use crate::db::pool::create_pool;// Import the connection pool 
use crate::controllers::db_controller::test_db;
use crate::controllers::user_controller::user_routes;// Import user routes from user controller
use crate::db::pool::migrate_db; // Import the migrate_db function


#[tokio::main] // Indicates that the main function is an async function using tokio
async fn main() {
    // Load environment variables from .env file
    dotenv().ok();
    
    /*
    / Creating the Pool using SQLx
    / Creates the pool before building router
    / connects to db using the DATABASE_URL from environment and returns a PgPool
    */
    let pool = create_pool().await;

   // Collect command line arguments
   let args: Vec<String> = env::args().collect();

   // If the user passes the 'migrate' argument, run the migrations
   if args.len() > 1 && args[1] == "migrate" {
       println!("Running migrations...");
       migrate_db(&pool).await;
   }

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
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
