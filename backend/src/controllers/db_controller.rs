use axum::{
    extract::Extension,
    response::Json,
};
use sqlx::Row; // Allows us to use the Row trait to get the result of a query

/*
/ Define the test_db function
/ This function is called when the /api/test-db route is hit
/ It returns a JSON object with a message
/ Should most likely be moved to a controller file later but its here for now
*/
pub async fn test_db(
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