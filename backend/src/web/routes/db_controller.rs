/*
/ src/controllers/db_controller.rs
/ Request Handlers
/
/ File containing various API Backend endpoints for manipulating the database and environment
/
/ API Summary:
/ api_test_db   GET    /test    - Test The Database Connection
/ api_wipe_db   GET    /wipe    - Wipe The Database If Secret Code Matches
/
*/
use axum::{
    extract::{Extension, Query},
    response::Json,
    routing::get,
    Router,
};
use serde_json::{json, Value};
use std::{fs, path::PathBuf, time::Duration};
use tokio::time;
use reqwest::Client;

use crate::models::db::WipeParams;
use crate::{Error, Result};

/// GET handler for testing the database connection.
/// Accessible via: GET /api/db/test
/// Test: test_environment.rs/test_database()
/// Frontend: Not directly called from frontend
pub async fn api_db_test(Extension(pool): Extension<sqlx::PgPool>) -> Result<Json<Value>> {
    println!("->> {:<12} - test_db", "HANDLER");

    // Run a simple query to ping the database.
    let result_row = sqlx::query("SELECT 1").fetch_one(&pool).await;

    match result_row {
        Ok(_) => {
            // Create Success
            let success = Json(json!({
                "result": {
                    "success": true
                }
            }));

            Ok(success)
        }
        Err(e) => {
            println!("Error connecting to database: {:?}", e);
            Err(Error::DatabaseConnectionError)
        }
    }
}

/// GET handler for resetting the database with a secret key.
/// Accessible via: GET /api/db/wipe?secret=secret_key
/// Test: test_environment.rs/test_reset_db()
/// Frontend: Not directly called from frontend
async fn api_db_reset(
    Extension(pool): Extension<sqlx::PgPool>,
    Query(params): Query<WipeParams>,
) -> Result<Json<Value>> {
    println!("->> {:<12} - wipe_db_method", "HANDLER");

    // Check for secret key needed to wipe db
    if params.secret != Some("secret_key".to_string()) {
        return Err(Error::MigrationKeyError);
    }

    // Read the migration script
    let migration_path = PathBuf::from("migrations/01_migration_script.sql");
    let migration_sql = fs::read_to_string(migration_path).map_err(|e| {
        println!("Error reading migration file: {:?}", e);
        Error::MigrationExecutionError
    })?;

    // Execute each statement in the migration script
    let statements: Vec<&str> = migration_sql
        .split(';')
        .filter(|s| !s.trim().is_empty())
        .collect();

    for (i, statement) in statements.iter().enumerate() {
        sqlx::query(statement).execute(&pool).await.map_err(|e| {
            println!("Error executing statement {}: {:?}", i + 1, e);
            Error::MigrationExecutionError
        })?;
    }

    Ok(Json(json!({
        "result": {
            "success": true,
            "message": "Database wiped successfully"
        }
    })))
}

async fn send_periodic_request(pool: sqlx::PgPool) {
    let client = Client::new();
    loop {
        // Send the HTTP GET request
        match client.get("https://vynn.app").send().await {
            Ok(response) => {
                println!("Response: {:?}", response.status());
            }
            Err(e) => {
                println!("Error sending request: {:?}", e);
            }
        }

        // Run a simple query to ping the database.
        let result_row = sqlx::query("SELECT 1").fetch_one(&pool).await;

        match result_row {
            Ok(_) => {
                // Create Success
                println!("Database ping successful.");
            }
            Err(e) => {
                println!("Error connecting to database: {:?}", e);
            }
        }
        time::sleep(Duration::from_secs(120)).await; // Sleep for 2 minutes
    }
}

pub fn db_routes(pool: sqlx::PgPool) -> Router {
    // Start the periodic request in a separate task
    let pool_clone = pool.clone(); // Clone the pool to pass to the task
    tokio::spawn(send_periodic_request(pool_clone));

    Router::new()
        .route("/test", get(api_db_test))
        .route("/reset", get(api_db_reset))
}
