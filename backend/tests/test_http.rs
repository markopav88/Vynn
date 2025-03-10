/*
 / HOW TO USE BACKEND TESTS
 / ENSURE WATCH IS INSTALLED '$ cargo install cargo-watch --locked'
 / In Terminal 1: 'cargo watch -q -c -w src/ -x run'
 / In Terminal 2: 'cargo watch -q -c -w tests/ -x "test -q test_http -- --nocapture"'
 / Now you can see LIVE Updates of API calls
*/

#![allow(unused)]

use anyhow::{Ok, Result};
use axum::http::response;
use httpc_test::Client;
use serde_json::json;

#[tokio::test]
async fn test_http() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:3001")?;

    println!("\n===== RUNNING API TESTS =====\n");

    // Run all tests and collect results
    let db_result = test_database(&hc).await;
    let user_result = test_create_user(&hc).await;
    let good_login_result = test_good_login(&hc).await;
    let bad_login_result = test_bad_login(&hc).await;
    let test_get_user_result = test_get_user(&hc).await;
    let wipe_db_result = test_wipe_db(&hc).await;

    // Print summary
    println!("\n===== TEST RESULTS =====");
    println!("Database Connection: {}", result_to_string(&db_result));
    println!("User Creation: {}", result_to_string(&user_result));
    println!("Good Login: {}", result_to_string(&good_login_result));
    println!("Bad Login: {}", result_to_string(&bad_login_result));
    println!("Get User: {}", result_to_string(&test_get_user_result));
    println!("Wipe Database: {}", result_to_string(&wipe_db_result));
    println!("=====================\n");

    Ok(())
}

fn result_to_string(result: &anyhow::Result<()>) -> &str {
    if result.is_ok() {
        "PASSED"
    } else {
        "FAILED"
    }
}

async fn test_create_user(hc: &Client) -> Result<()> {
    println!("TEST - User Creation");

    // Create user
    let create_response = hc
        .do_post(
            "/api/users",
            json!({
                "name": "Test User",
                "email": "testcreate@example.com",
                "password": "password123"
            }),
        )
        .await?;

    create_response.print().await?;

    // Check if the creation was successful (status code 2xx)
    if !create_response.status().is_success() {
        return Err(anyhow::anyhow!(
            "User creation failed with status: {}",
            create_response.status()
        ));
    }

    // Extract the user ID from the response body
    let body = create_response
        .json_body()
        .expect("Failed to get JSON body");
    let user_id = body["id"].as_i64().unwrap_or(1);

    // Try to get the user with the extracted ID
    let get_response = hc.do_get(&format!("/api/users/{}", user_id)).await?;
    get_response.print().await?;

    // Check if the get request was successful
    if !get_response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to get created user"));
    }

    Ok(())
}

async fn test_database(hc: &Client) -> Result<()> {
    print!("TEST - Database Connection");
    let response = hc.do_get("/api/test-db").await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Test Database failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_good_login(hc: &Client) -> Result<()> {
    print!("TEST - Good Login");
    let response = hc
        .do_post(
            "/api/login",
            json!({
                "email": "Hello@gmail.com",
                "password": "test"
            }),
        )
        .await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Login failed with status: {}",
            response.status()
        ));
    }

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

async fn test_bad_login(hc: &Client) -> Result<()> {
    print!("TEST - Bad Login");
    let response = hc
        .do_post(
            "/api/login",
            json!({
                "email": "Hell2o@gmail.com",
                "password": "test"
            }),
        )
        .await?;
    response.print().await?;

    if response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Bad login Succeed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_wipe_db(hc: &Client) -> Result<()> {
    print!("TEST - Wipe Database");
    let response = hc.do_get("/api/wipe-db?secret=secret_key").await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Wipe DB failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_get_user(hc: &Client) -> Result<()> {
    print!("TEST - Get User");
    let response = hc.do_get("/api/users/1").await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Get User failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}
