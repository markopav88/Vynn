/*
 / HOW TO USE BACKEND TESTS
 / ENSURE WATCH IS INSTALLED $ cargo install cargo-watch --locked
 / In Terminal 1: 'cargo watch -q -c -w src/ -x run'
 / In Terminal 2: 'cargo watch -q -c -w tests/ -x "test -q test_http -- --nocapture"'
 / Now you can see LIVE Updates of API calls
*/

#![allow(unused)]

use anyhow::Result;
use serde_json::json;
use httpc_test::Client;

#[tokio::test]
async fn test_http() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:3001")?;

    println!("\n===== RUNNING API TESTS =====\n");

    // Run all tests and collect results
    let db_result = test_database(&hc).await;
    let user_result = test_create_user(&hc).await;
    let good_login_result = test_good_login(&hc).await;
    let bad_login_result = test_bad_login(&hc).await;
    
    // Print summary
    println!("\n===== TEST RESULTS =====");
    println!("Database Connection: {}", result_to_string(&db_result));
    println!("User Creation: {}", result_to_string(&user_result));
    println!("Good Login: {}", result_to_string(&good_login_result));
    println!("Bad Login: {}", result_to_string(&bad_login_result));
    println!("=====================\n");

    Ok(())
}

fn result_to_string<T>(result: &Result<T>) -> &str {
    match result {
        Ok(_) => "✅ PASSED",
        Err(_) => "❌ FAILED",
    }
}

async fn test_create_user(hc: &Client) -> Result<()> {
    println!("TEST - User Creation");
    
    // Create user
    let create_response = hc.do_post(
        "/api/users",
        json!({
            "name": "Test User",
            "email": "test@example.com",
            "password": "password123"
        })
    ).await?;
    
    create_response.print().await?;
    
    // Check if the creation was successful (status code 2xx)
    if !create_response.status().is_success() {
        return Err(anyhow::anyhow!("User creation failed with status: {}", create_response.status()));
    }
    
    // Try to get the user
    let get_response = hc.do_get("/api/users/1").await?;
    get_response.print().await?;
    
    // Check if the get request was successful
    if !get_response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to get created user"));
    }
    
    Ok(())
}

async fn test_database(hc: &Client) -> Result<()> {
    print!("TEST - Database Connection");
    hc.do_get("/api/test-db").await?.print().await?;

    Ok(())
}

async fn test_good_login(hc: &Client) -> Result<()> {
    print!("TEST - Good Login");
    let req_login = hc.do_post("/api/login", json!({
        "email": "Hello@gmail.com",
        "password": "test"
    }));
    req_login.await?.print().await?;

    Ok(())
}

async fn trigger_fallback(hc: &Client) -> Result<()> {
    print!("TEST - Trigger Fallback Route");
    hc.do_get("/src/main.rs").await?.print().await?;
    
    Ok(())
}

async fn test_bad_login(hc: &Client) -> Result<()> {
    print!("TEST - Bad Login");
    let req_login = hc.do_post("/api/login", json!({
        "email": "Hell2o@gmail.com",
        "password": "test"
    }));
    req_login.await?.print().await?;

    Ok(())
}