#![allow(unused)]

use anyhow::{anyhow, Result};
use backend::result_to_string;
use chrono::Utc;
use httpc_test::Client;
use serde_json::json;

#[tokio::test]
async fn test_preferences() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:3001")?;

    println!("\n===== RUNNING PREFERENCES API TESTS =====\n");

    // Run all tests and collect results
    let login_result = test_good_login(&hc).await;
    let get_all_prefs = test_get_all_preferences(&hc).await;
    let get_specific_pref = test_get_specific_preference(&hc).await;
    let update_specific_pref = test_update_specific_preference(&hc).await;
    let reset_specific_pref = test_reset_specific_preference(&hc).await;
    let reset_all_prefs = test_reset_all_preferences(&hc).await;
    let upload_bg = test_upload_background_image(&hc).await;
    let get_bg = test_get_background_image(&hc).await;
    let delete_bg = test_delete_background_image(&hc).await;
    let get_default_bg = test_get_default_background_image(&hc).await;
    let reset_db = backend::test_reset_db(&hc).await;

    // Print summary
    println!("\n======== TEST RESULTS ========");
    println!("Login as User 1\t\t{}", result_to_string(&login_result));
    println!("Get All Preferences\t{}", result_to_string(&get_all_prefs));
    println!("Get Specific Pref\t{}", result_to_string(&get_specific_pref));
    println!("Update Specific Pref\t{}", result_to_string(&update_specific_pref));
    println!("Reset Specific Pref\t{}", result_to_string(&reset_specific_pref));
    println!("Reset All Prefs\t\t{}", result_to_string(&reset_all_prefs));
    println!("Upload Background\t{}", result_to_string(&upload_bg));
    println!("Get Background\t\t{}", result_to_string(&get_bg));
    println!("Delete Background\t{}", result_to_string(&delete_bg));
    println!("Get Default BG\t\t{}", result_to_string(&get_default_bg));
    println!("Reset Database\t\t{}", result_to_string(&reset_db));
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

async fn test_get_all_preferences(hc: &Client) -> Result<()> {
    println!("TEST - Get All Preferences");
    
    let response = hc.do_get("/api/preference").await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Get all preferences failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_get_specific_preference(hc: &Client) -> Result<()> {
    println!("TEST - Get Specific Preference");
    
    // Test with preference ID 1 (assuming it exists)
    let response = hc.do_get("/api/preference/1").await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Get specific preference failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_update_specific_preference(hc: &Client) -> Result<()> {
    println!("TEST - Update Specific Preference");

    let preference = json!({
        "preference_value": "dark"
    });

    let response = hc.do_put("/api/preference/1", preference).await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Update specific preference failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_reset_specific_preference(hc: &Client) -> Result<()> {
    println!("TEST - Reset Specific Preference");

    let response = hc.do_delete("/api/preference/1").await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Reset specific preference failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_reset_all_preferences(hc: &Client) -> Result<()> {
    println!("TEST - Reset All Preferences");

    let response = hc.do_delete("/api/preference").await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Reset all preferences failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_upload_background_image(hc: &Client) -> Result<()> {
    println!("TEST - Upload Background Image");

    // Note: This is a simplified test. In reality, we'd need to handle multipart form data
    let response = hc.do_post("/api/preference/background", json!({})).await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Upload background image failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_get_background_image(hc: &Client) -> Result<()> {
    println!("TEST - Get Background Image");

    let response = hc.do_get("/api/preference/background").await?;
    response.print().await?;

    if !response.status().is_success() && response.status().as_u16() != 204 {
        return Err(anyhow::anyhow!(
            "Get background image failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_delete_background_image(hc: &Client) -> Result<()> {
    println!("TEST - Delete Background Image");

    let response = hc.do_delete("/api/preference/background").await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Delete background image failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_get_default_background_image(hc: &Client) -> Result<()> {
    println!("TEST - Get Default Background Image");

    let response = hc.do_get("/api/preference/default-background").await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Get default background image failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}
