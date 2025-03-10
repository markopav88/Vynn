use axum::{
    extract::Extension,
    response::Json,
};
use sqlx::Row; // Allows us to use the Row trait to get the result of a query
use crate::{Error, Result};
use serde_json::{Value, json};

/*
/ Define the test_db function
/ This function is called when the /api/test-db route is hit
/ It returns a JSON object with a message
*/
pub async fn test_db(
    Extension(pool): Extension<sqlx::PgPool>,
) -> Result<Json<Value>> {
    // Run a simple query to test the database connection.
    let result_row = sqlx::query!("SELECT 1 as number")
        .fetch_one(&pool)
        .await;

    match result_row {
        Ok(record) => {
            // 
            let number: i32 = result_row.try_get("number")
        .expect("Failed to get column 'number'");
        }
        Err(_) => {
            println!("Error connecting to database: {:?}", e);
            Err(Error::DatabaseConnectionError)
        }
    }

    

    // Create Success
    let success = Json(json!({
        "result": {
            "success": number
        }
    }));

    Ok(success)
}