// Global Defined Functions
use anyhow::Result;
use httpc_test::Client;

pub fn result_to_string(result: &anyhow::Result<()>) -> &str {
    if result.is_ok() {
        "PASSED"
    } else {
        "FAILED"
    }
}

pub async fn test_wipe_db(hc: &Client) -> Result<()> {
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