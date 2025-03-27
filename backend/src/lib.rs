// Global Defined Functions
use anyhow::Result;
use httpc_test::Client;
use tower_cookies::Cookies;

pub fn result_to_string(result: &anyhow::Result<()>) -> &str {
    if result.is_ok() {
        "PASSED"
    } else {
        "FAILED"
    }
}

pub async fn test_wipe_db(hc: &Client) -> Result<()> {
    print!("TEST - Wipe Database");
    let response = hc.do_get("/api/db/wipe?secret=secret_key").await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Wipe DB failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

// Helper function to extract user ID from auth cookie
pub fn get_user_id_from_cookie(cookies: &Cookies) -> Option<i32> {
    cookies.get("auth-token").and_then(|cookie| {
        let value = cookie.value();
        // Parse user ID from cookie value (format: "user-{id}.exp.sign")
        value
            .strip_prefix("user-")
            .and_then(|s| s.split('.').next())
            .and_then(|id_str| id_str.parse::<i32>().ok())
    })
}