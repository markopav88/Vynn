/*
 / HOW TO USE BACKEND TESTS
 / ENSURE WATCH IS INSTALLED '$ cargo install cargo-watch --locked'
 / In Terminal 1: 'cargo watch -q -c -w src/ -x run'
 / In Terminal 2: 'cargo watch -q -c -w tests/ -x "test -q test_testname -- --nocapture"'
 / Now you can see LIVE Updates of API calls
*/

/*
 / Keybinding API Tests
 / Run with: cargo test -q test_keybindings -- --nocapture
*/

#![allow(unused)]

use anyhow::Result;
use backend::result_to_string;
use httpc_test::Client;
use serde_json::json;

#[tokio::test]
async fn test_keybindings() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:3001")?;

    println!("\n===== RUNNING KEYBINDING API TESTS =====\n");

    // Run all tests and collect results
    let login_result = test_good_login(&hc).await;
    let get_all_commands_result = test_get_all_commands(&hc).await;
    let get_all_keybindings_result = test_get_all_keybindings(&hc).await;
    let add_update_keybinding_result = test_add_update_keybinding(&hc).await;
    let delete_keybinding_result = test_delete_keybinding(&hc).await;
    let reset_db = backend::test_reset_db(&hc).await;

    // Print summary
    println!("\n======== TEST RESULTS ========");
    println!("Login as User 1:\t{}", result_to_string(&login_result));
    println!("Get All Commands:\t{}", result_to_string(&get_all_commands_result));
    println!("Get All Keybindings:\t{}", result_to_string(&get_all_keybindings_result));
    println!("Add/Update Keybinding:\t{}", result_to_string(&add_update_keybinding_result));
    println!("Delete Keybinding:\t{}", result_to_string(&delete_keybinding_result));
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

async fn test_get_all_commands(hc: &Client) -> Result<()> {
    println!("TEST - Get All Commands");
    
    let response = hc.do_get("/api/command/default").await?;
    response.print().await?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Get all commands failed with status: {}",
            response.status()
        ));
    }
    
    Ok(())
}

async fn test_get_all_keybindings(hc: &Client) -> Result<()> {
    println!("TEST - Get All Keybindings");
    
    let response = hc.do_get("/api/command").await?;
    response.print().await?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Get all keybindings failed with status: {}",
            response.status()
        ));
    }
    
    Ok(())
}

async fn test_add_update_keybinding(hc: &Client) -> Result<()> {
    println!("TEST - Add/Update Keybinding");
    
    // Update keybinding for command ID 1 (Bold Selected)
    let response = hc
        .do_put(
            "/api/command/1",
            json!({
                "keybinding": "Ctrl+Shift+B"
            }),
        )
        .await?;
    
    response.print().await?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Add/Update keybinding failed with status: {}",
            response.status()
        ));
    }
    
    Ok(())
}

async fn test_delete_keybinding(hc: &Client) -> Result<()> {
    println!("TEST - Delete Keybinding");
    
    // Delete keybinding for command ID 1 (Bold Selected)
    let response = hc.do_delete("/api/command/1").await?;
    response.print().await?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Delete keybinding failed with status: {}",
            response.status()
        ));
    }
    
    Ok(())
}
