/*
 / HOW TO USE BACKEND TESTS
 / ENSURE WATCH IS INSTALLED '$ cargo install cargo-watch --locked'
 / In Terminal 1: 'cargo watch -q -c -w src/ -x run'
 / In Terminal 2: 'cargo watch -q -c -w tests/ -x "test -q test_testname -- --nocapture"'
 / Now you can see LIVE Updates of API calls
*/

#![allow(unused)]

use anyhow::Result;
use axum::http::response;
use backend::result_to_string;
use httpc_test::Client;
use serde_json::json;

#[tokio::test]
async fn test_environment() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:3001")?;

    println!("\n===== RUNNING DATABASE API TESTS =====\n");

    // Run all tests and collect results
    let db_result = test_database(&hc).await;
    let fallback_result = trigger_fallback(&hc).await;
    let db_reset = backend::test_reset_db(&hc).await;

    // Print summary
    println!("\n==== TEST RESULTS ====");
    println!("Database Query:\t{}", result_to_string(&db_result));
    println!("Test Fallback:\t{}", result_to_string(&fallback_result));
    println!("Reset Database:\t{}", result_to_string(&db_reset));
    println!("======================\n");

    Ok(())
}

async fn trigger_fallback(hc: &Client) -> Result<()> {
    print!("TEST - Trigger Fallback Route");
    let response = hc.do_get("/src/main.rs").await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Fallback failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_database(hc: &Client) -> Result<()> {
    print!("TEST - Database Connection");
    let response = hc.do_get("/api/db/test").await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Test Database failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}
