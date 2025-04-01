/*
 / HOW TO USE BACKEND TESTS
 / ENSURE WATCH IS INSTALLED '$ cargo install cargo-watch --locked'
 / In Terminal 1: 'cargo watch -q -c -w src/ -x run'
 / In Terminal 2: 'cargo watch -q -c -w tests/ -x "test -q test_testname -- --nocapture"'
 / Now you can see LIVE Updates of API calls
*/

/*
 / Document API Tests
 / Run with: cargo test -q test_documents -- --nocapture
*/

#![allow(unused)]

use std::result;

use anyhow::{anyhow, Result};
use backend::result_to_string;
use chrono::Utc;
use httpc_test::Client;
use serde_json::json;

#[tokio::test]
async fn test_documents() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:3001")?;

    println!("\n===== RUNNING DOCUMENT API TESTS =====\n");

    // Run all tests and collect results
    let login_result = test_good_login(&hc).await;
    let create_result = test_create_document(&hc).await;
    let get_result = test_get_document(&hc).await;
    let get_docs_results = test_get_all_doc(&hc).await;
    let get_project_from_doc = test_get_project_from_document(&hc).await;
    let update_result = test_update_document(&hc).await;
    let add_permissions = test_add_permissions(&hc).await;
    let upd_perm = test_update_permissions(&hc).await;
    let get_perm = test_get_permissions(&hc).await;
    let delete_result = test_delete_document(&hc).await;
    let get_bad_result = test_get_document(&hc).await;
    let reset_db = backend::test_reset_db(&hc).await;

    // Print summary
    println!("\n======== TEST RESULTS ========");
    println!("Login as User 1\t\t{}", result_to_string(&login_result));
    println!("Create Document:\t{}", result_to_string(&create_result));
    println!("Get Document:\t\t{}", result_to_string(&get_result));
    println!("Update Document:\t{}", result_to_string(&update_result));
    println!("Add Permissions:\t{}", result_to_string(&add_permissions));
    println!("Update Permissions:\t{}", result_to_string(&upd_perm));
    println!("Get Users Permissions:\t{}", result_to_string(&get_perm));
    println!("Delete Document\t\t{}", result_to_string(&delete_result));
    println!("Get Bad Document:\t{}", result_to_string(&get_result));
    println!("Reset Database:\t\t{}", result_to_string(&reset_db));
    println!("==============================\n");

    Ok(())
}

// Test login to set the auth cookie and allow for validation
pub async fn test_good_login(hc: &Client) -> Result<()> {
    print!("TEST - Good Login");
    let response = hc
        .do_post(
            "/api/users/login",
            json!({
                "email": "CFdefence@gmail.com",
                "password": "MyPassword"
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

async fn test_create_document(hc: &Client) -> Result<()> {
    println!("TEST - Document Creation");

    // Create document
    let now = Utc::now().naive_utc();
    let create_response = hc
        .do_post(
            "/api/document",
            json!({
                "name": "Test Document",
                "content": "This is a test document content",
                "created_at": now,
                "updated_at": now
            }),
        )
        .await?;

    create_response.print().await?;

    // Check if the creation was successful (status code 2xx)
    if !create_response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Document creation failed with status: {}",
            create_response.status()
        ));
    }

    // Extract the document ID from the response body
    let body = create_response
        .json_body()
        .expect("Failed to get JSON body");
    let document_id = body["id"].as_i64().unwrap_or(1);
    print!("document_id {}", document_id);

    // Try to get the document with the extracted ID
    let get_response = hc.do_get(&format!("/api/document/{}", document_id)).await?;
    get_response.print().await?;

    // Check if the get request was successful
    if !get_response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to get created document"));
    }

    Ok(())
}

async fn test_get_document(hc: &Client) -> Result<()> {
    println!("TEST - Get Document");

    // Try to get document with ID 3 (just created)
    let response = hc.do_get("/api/document/2").await?;
    response.print().await?;

    // Check if the get request was successful
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Get Document failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_get_all_doc(hc: &Client) -> Result<()> {
    println!("TEST - Get All Documents");

    // Get all documents signed in user has
    let get_response = hc.do_get("/api/document").await?;
    get_response.print().await?;

    // Check if the get request was successful
    if !get_response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Get All Documents failed with status: {}",
            get_response.status()
        ));
    }

    Ok(())
}

async fn test_update_document(hc: &Client) -> Result<()> {
    println!("TEST - Update Document");

    // generate new updated_at time
    let now = Utc::now().naive_utc();

    // Now update the document we just created
    let update_response = hc
        .do_put(
            &format!("/api/document/2"),
            json!({
                "name": "Updated Test Document",
                "content": "This document has been updated",
                "updated_at": now
            }),
        )
        .await?;

    update_response.print().await?;

    // Check if the update was successful
    if !update_response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Update Document failed with status: {}",
            update_response.status()
        ));
    }

    Ok(())
}

async fn test_get_project_from_document(hc: &Client) -> Result<()> {
    println!("TEST - Get Project From Document");

    // lets use document 1 because user 1 has access
    let document_id = 1;

    // call get API
    let get_response = hc.do_get(&format!("/api/{}/project", document_id)).await?;
    get_response.print().await;

    // check the response status
    if !get_response.status().is_success() {
        return Err(anyhow!(
            "Get Project From Document ID failed with status: {}",
            get_response.status()
        ));
    }

    Ok(())
}

async fn test_delete_document(hc: &Client) -> Result<()> {
    println!("TEST - Delete Document");

    // Now delete the document we just created
    let delete_response = hc.do_delete(&format!("/api/document/2")).await?;

    delete_response.print().await?;

    // Check if the update was successful
    if !delete_response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Delete Document failed with status: {}",
            delete_response.status()
        ));
    }

    Ok(())
}

async fn test_add_permissions(hc: &Client) -> Result<()> {
    println!("TEST - Add Document Permissions");

    // Add permissions for user 2 on document 1 (created in previous tests)
    let add_perm_response = hc
        .do_post(
            "/api/document/1/permissions",
            json!({
                "user_id": 2,
                "role": "editor"
            }),
        )
        .await?;

    add_perm_response.print().await?;

    if !add_perm_response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Add permissions failed with status: {}",
            add_perm_response.status()
        ));
    }

    Ok(())
}

async fn test_get_permissions(hc: &Client) -> Result<()> {
    println!("TEST - Get Document Permissions");

    // Get permissions for document 1
    let get_perm_response = hc.do_get("/api/document/1/permissions").await?;

    get_perm_response.print().await?;

    if !get_perm_response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Get permissions failed with status: {}",
            get_perm_response.status()
        ));
    }

    Ok(())
}

async fn test_update_permissions(hc: &Client) -> Result<()> {
    println!("TEST - Update Document Permissions");

    // Update permission of user_id 2 which should always exist in the database
    let update_response = hc
        .do_put(
            "/api/document/1/permissions",
            json!({
                "user_id": 2,
                "role": "viewer"
            }),
        )
        .await?;

    update_response.print().await?;

    if !update_response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to update permission"));
    }

    Ok(())
}

async fn test_remove_permissions(hc: &Client) -> Result<()> {
    println!("TEST - Remove Document Permission");

    // Remove permissions for user 2 on document 1
    let remove_perm_response = hc.do_delete("/api/document/1/permissions/2").await?;

    remove_perm_response.print().await?;

    if !remove_perm_response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Remove permission failed with status: {}",
            remove_perm_response.status()
        ));
    }

    Ok(())
}
