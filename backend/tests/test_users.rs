#![allow(unused)]

use std::any;
use std::fs;
use std::path::Path;

use anyhow::Result;
use axum::http::response;
use backend::result_to_string;
use httpc_test::Client;
use serde_json::json;

#[tokio::test]
async fn test_users() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:3001")?;

    println!("\n===== RUNNING USER API TESTS =====\n");

    // Run all tests and collect results
    let create_user_result = test_create_user(&hc).await;
    let good_login_result = test_good_login(&hc).await;
    let bad_login_result = test_bad_login(&hc).await;
    let update_user_result = test_update_user(&hc).await;
    let get_user_result = test_get_user(&hc).await;
    let get_current_user_result = test_get_current_user(&hc).await;
    let check_auth_result = test_check_auth(&hc).await;
    let upload_image_result = test_upload_profile_image(&hc).await;
    let get_image_result = test_get_profile_image(&hc, 1).await; // Assuming user 1 exists
    let logout_result = test_logout(&hc).await;
    let reset_db_result = backend::test_reset_db(&hc).await;

    // Print summary
    println!("\n==== TEST RESULTS ====");
    println!("Create User:\t{}", result_to_string(&create_user_result));
    println!("Good Login:\t{}", result_to_string(&good_login_result));
    println!("Bad Login:\t{}", result_to_string(&bad_login_result));
    println!("Update User:\t{}", result_to_string(&update_user_result));
    println!("Get User:\t{}", result_to_string(&get_user_result));
    println!("Get Current User:\t{}", result_to_string(&get_current_user_result));
    println!("Check Auth:\t\t{}", result_to_string(&check_auth_result));
    println!("Upload Image:\t{}", result_to_string(&upload_image_result));
    println!("Get Image:\t\t{}", result_to_string(&get_image_result));
    println!("Logout:\t\t{}", result_to_string(&logout_result));
    println!("Reset Database:\t{}", result_to_string(&reset_db_result));
    println!("======================\n");

    Ok(())
}

async fn test_create_user(hc: &Client) -> Result<()> {
    println!("TEST - User Creation");
    let response = hc
        .do_post(
            "/api/users",
            json!({
                    "name": "Test User",
                    "email": "testcreate@example.com",
                    "password": "password123"
            }),
        )
        .await?;

    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "User creation failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

pub async fn test_good_login(hc: &Client) -> Result<()> {
    print!("TEST - Good Login");
    let response = hc
        .do_post(
            "/api/users/login",
            json!({
                "email": "testcreate@example.com",
                "password": "password123"
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

async fn test_update_user(hc: &Client) -> Result<()> {
    print!("TEST - Update User");
    // attempt api call on user 1
    let response = hc
        .do_put(
            "/api/users/update",
            json!({
                "name": "updated_name",
                "email": "updated_email",
                "password": "updated_password",

            }),
        )
        .await?;

    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "User creation failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_bad_login(hc: &Client) -> Result<()> {
    print!("TEST - Bad Login");
    let response = hc
        .do_post(
            "/api/users/login",
            json!({
                "email": "Hell2o@gmail.com",
                "password": "bad password"
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

async fn test_get_user(hc: &Client) -> Result<()> {
    print!("TEST - Get User");

    // Get user id 3
    let response = hc.do_get("/api/users/3").await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Get User failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_logout(hc: &Client) -> Result<()> {
    print!("TEST - Logout");
    let response = hc.do_get("/api/users/logout").await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Logout failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_get_current_user(hc: &Client) -> Result<()> {
    println!("TEST - Get Current User");
    
    // First, ensure we're logged in
    let login_response = hc
        .do_post(
            "/api/users/login",
            json!({
                "email": "testcreate@example.com",
                "password": "password123"
            }),
        )
        .await?;
    
    if !login_response.status().is_success() {
        return Err(anyhow::anyhow!("Could not login for current user test"));
    }
    
    // Now try to get current user
    let response = hc.do_get("/api/users/current").await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Get Current User failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_check_auth(hc: &Client) -> Result<()> {
    println!("TEST - Check Auth");
    let res = hc.do_get("/api/users/check-auth").await?;
    res.print().await?;
    if !res.status().is_success() {
        return Err(anyhow::anyhow!("Check Auth failed"));
    }
    Ok(())
}

async fn test_upload_profile_image(hc: &Client) -> Result<()> {
    println!("TEST - Upload Profile Image");
    // TODO: Implement actual file upload logic here
    // This requires constructing a multipart/form-data request which httpc-test might need setup for.
    // Placeholder: Check if endpoint exists (expects 400 Bad Request without proper data)
    let res = hc.do_post("/api/users/profile-image", json!({})).await?;
    res.print().await?;
    // Expecting a failure status code without actual data, but the endpoint should exist.
    // If it's 404, the route is missing. If 400/422, route exists but needs data.
    if res.status().as_u16() == 404 {
        return Err(anyhow::anyhow!("Upload profile image endpoint not found"));
    }
    println!("-> Endpoint likely exists (status: {})", res.status());
    Ok(())
}

async fn test_get_profile_image(hc: &Client, user_id: i32) -> Result<()> {
    println!("TEST - Get Profile Image (User: {})", user_id);
    let res = hc.do_get(&format!("/api/users/{}/profile-image", user_id)).await?;
    // Don't print body (it's binary image data)
    println!("-> Status: {}", res.status());
    // Status could be 200 OK (image found) or 404 Not Found (no image or user not found)
    if !res.status().is_success() && res.status().as_u16() != 404 {
        return Err(anyhow::anyhow!(
            "Get profile image failed with unexpected status: {}",
            res.status()
        ));
    }
    Ok(())
}