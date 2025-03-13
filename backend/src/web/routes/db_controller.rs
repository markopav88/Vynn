use axum::{
    Router,
    extract::{Extension, Query},
    response::Json,
    routing::get,
};
use serde::Deserialize;
use crate::{Error, Result};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

#[derive(Deserialize)]
struct WipeParams {
    secret: Option<String>,
}

/*
/ Define the test_db function
/ This function is called when the /api/test-db route is hit
/ It returns a JSON object with a message
*/
pub async fn test_db(
    Extension(pool): Extension<sqlx::PgPool>
) -> Result<Json<Value>> {
    println!("->> {:<12} - test_db", "HANDLER");

    // Run a simple query to test the database connection.
    let result_row = sqlx::query!("SELECT 1 as number")
        .fetch_one(&pool)
        .await;

    match result_row {
        Ok(record) => {
            let number: i32 = record.number.unwrap_or(0);

            // Create Success
            let success = Json(json!({
                "result": {
                    "success": number
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

/*
/ Define the wipe_db function
/ This function is called when the /api/wipe-db route is hit and a special key is passed
/ It will execute the migration script which will reset the database
/ It returns a JSON object with a message
*/
async fn wipe_db(
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
    let migration_sql = fs::read_to_string(migration_path)
        .map_err(|e| {
            println!("Error reading migration file: {:?}", e);
            Error::MigrationExecutionError
        })?;

    // Execute each statement in the migration script
    let statements: Vec<&str> = migration_sql
        .split(';')
        .filter(|s| !s.trim().is_empty())
        .collect();

    for (i, statement) in statements.iter().enumerate() {
        sqlx::query(statement)
            .execute(&pool)
            .await
            .map_err(|e| {
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

pub fn db_routes() -> Router {
    Router::new()
        .route("/api/test-db", get(test_db))
        .route("/api/wipe-db",get(wipe_db))
}