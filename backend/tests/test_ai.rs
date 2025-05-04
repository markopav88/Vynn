#![allow(unused)]

use anyhow::{anyhow, Result};
use httpc_test::Client;
use serde_json::{json, Value};
use backend::{result_to_string, test_reset_db, get_user_id_from_cookie};
use chrono::Utc;

#[tokio::test]
async fn test_ai() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:3001")?;

    println!("\n===== RUNNING AI API TESTS =====\n");

    // --- Setup: Ensure user is logged in (similar to test_documents.rs) ---
    let login_result = test_good_login(&hc).await;
    let get_sessions_res = test_get_all_writing_sessions_success(&hc).await;
    let create_session_res = test_create_writing_session_success(&hc).await;
    let get_session_res = test_get_writing_session_success(&hc, 1).await; // Placeholder ID
    let send_message_res = test_send_writing_message_success(&hc, 1).await; // Placeholder ID
    let check_grammar_res = test_check_grammar_success(&hc).await;
    let summarize_res = test_summarize_success(&hc).await;
    let rephrase_res = test_rephrase_success(&hc).await;
    let expand_res = test_expand_success(&hc).await;
    let shrink_res = test_shrink_success(&hc).await;
    let rewrite_res = test_rewrite_success(&hc).await;
    let fact_check_res = test_fact_check_success(&hc).await;
    let spell_check_res = test_spell_check_success(&hc).await;
    let delete_session_res = test_delete_writing_session_success(&hc, 1).await; // Placeholder ID

    // Print summary
    println!("\n======== TEST RESULTS ========");
    println!("Login as User 1\t\t{}", result_to_string(&login_result));
    println!("Get All Sessions:\t{}", result_to_string(&get_sessions_res));
    println!("Create Session:\t\t{}", result_to_string(&create_session_res.map(|_| ())));
    println!("Get Session:\t\t{}", result_to_string(&get_session_res));
    println!("Send Message:\t\t{}", result_to_string(&send_message_res));
    println!("Check Grammar:\t\t{}", result_to_string(&check_grammar_res));
    println!("Summarize Text:\t{}", result_to_string(&summarize_res));
    println!("Rephrase Text:\t{}", result_to_string(&rephrase_res));
    println!("Expand Text:\t\t{}", result_to_string(&expand_res));
    println!("Shrink Text:\t\t{}", result_to_string(&shrink_res));
    println!("Rewrite Text:\t\t{}", result_to_string(&rewrite_res));
    println!("Fact Check Text:\t{}", result_to_string(&fact_check_res));
    println!("Spell Check Text:\t{}", result_to_string(&spell_check_res));
    println!("Delete Session:\t{}", result_to_string(&delete_session_res));
    // if reset_db_res.is_ok() { println!("Reset Database:\t\tPASSED"); } else { println!("Reset Database:\t\tFAILED"); }
    println!("==============================\n");

    Ok(())
}

// Helper: Login function (copied from test_documents.rs for self-containment)
async fn test_good_login(hc: &Client) -> Result<()> {
    print!("TEST - Good Login (AI Setup): ");
    let response = hc
        .do_post(
            "/api/users/login",
            json!({
                "email": "CFdefence@gmail.com",
                "password": "MyPassword"
            }),
        )
        .await?;
    // response.print().await?; // Optional: reduce noise
    if !response.status().is_success() {
        println!("FAILED");
        return Err(anyhow!("Login failed with status: {}", response.status()));
    }
    println!("PASSED");
    Ok(())
}

async fn test_get_all_writing_sessions_success(hc: &Client) -> Result<()> {
    print!("TEST - Get All Writing Sessions: ");
    let res = hc.do_get("/api/writing-assistant").await?;
    if !res.status().is_success() {
        println!("FAILED");
        return Err(anyhow!("Status: {}", res.status()));
    }
    println!("PASSED ({})", res.status());
    Ok(())
}

async fn test_create_writing_session_success(hc: &Client) -> Result<i32> { // Return ID for later use
    print!("TEST - Create Writing Session: ");
    let payload = json!({
        "document_id": null, // Or provide a valid one if needed
        "title": "AI Test Session"
    });
    let res = hc.do_post("/api/writing-assistant", payload).await?;
    if !res.status().is_success() {
        println!("FAILED");
        return Err(anyhow!("Status: {}", res.status()));
    }
    let body = res.json_body()?;
    let session_id = body["id"].as_i64().ok_or(anyhow!("Missing session ID"))? as i32;
    println!("PASSED ({}), ID: {}", res.status(), session_id);
    Ok(session_id)
}

async fn test_get_writing_session_success(hc: &Client, session_id: i32) -> Result<()> {
    print!("TEST - Get Writing Session (ID: {}): ", session_id);
    let res = hc.do_get(&format!("/api/writing-assistant/{}", session_id)).await?;
    if !res.status().is_success() {
        println!("FAILED");
        return Err(anyhow!("Status: {}", res.status()));
    }
    println!("PASSED ({})", res.status());
    Ok(())
}

async fn test_send_writing_message_success(hc: &Client, session_id: i32) -> Result<()> {
    print!("TEST - Send Writing Message (Session ID: {}): ", session_id);
    let payload = json!({
        "content": "Hello AI assistant!"
    });
    let res = hc.do_post(&format!("/api/writing-assistant/{}/message", session_id), payload).await?;
    if !res.status().is_success() {
        println!("FAILED");
        return Err(anyhow!("Status: {}", res.status()));
    }
    println!("PASSED ({})", res.status());
    Ok(())
}

async fn test_delete_writing_session_success(hc: &Client, session_id: i32) -> Result<()> {
    print!("TEST - Delete Writing Session (ID: {}): ", session_id);
    let res = hc.do_delete(&format!("/api/writing-assistant/{}", session_id)).await?;
    if !res.status().is_success() {
        println!("FAILED");
        return Err(anyhow!("Status: {}", res.status()));
    }
    println!("PASSED ({})", res.status());
    Ok(())
}

async fn test_check_grammar_success(hc: &Client) -> Result<()> {
    print!("TEST - Check Grammar: ");
    let payload = json!({ "content": "this sentence have bad grammer." });
    let res = hc.do_post("/api/writing-assistant/grammer", payload).await?;
    if !res.status().is_success() {
        println!("FAILED");
        return Err(anyhow!("Status: {}", res.status()));
    }
    println!("PASSED ({})", res.status());
    Ok(())
}

async fn test_summarize_success(hc: &Client) -> Result<()> {
    print!("TEST - Summarize Text: ");
    let payload = json!({ "content": "This is a long piece of text that needs to be summarized into a shorter form." });
    let res = hc.do_post("/api/writing-assistant/summarize", payload).await?;
     if !res.status().is_success() {
        println!("FAILED");
        return Err(anyhow!("Status: {}", res.status()));
    }
    println!("PASSED ({})", res.status());
    Ok(())
}

async fn test_rephrase_success(hc: &Client) -> Result<()> {
    print!("TEST - Rephrase Text: ");
    let payload = json!({ "content": "The text is needing rephrasing." });
    let res = hc.do_post("/api/writing-assistant/rephrase", payload).await?;
     if !res.status().is_success() {
        println!("FAILED");
        return Err(anyhow!("Status: {}", res.status()));
    }
    println!("PASSED ({})", res.status());
    Ok(())
}

async fn test_expand_success(hc: &Client) -> Result<()> {
    print!("TEST - Expand Text: ");
    let payload = json!({ "content": "Short idea." });
    let res = hc.do_post("/api/writing-assistant/expand", payload).await?;
     if !res.status().is_success() {
        println!("FAILED");
        return Err(anyhow!("Status: {}", res.status()));
    }
    println!("PASSED ({})", res.status());
    Ok(())
}

async fn test_shrink_success(hc: &Client) -> Result<()> {
    print!("TEST - Shrink Text: ");
    let payload = json!({ "content": "This is a very long and verbose explanation that could definitely be made more concise." });
    let res = hc.do_post("/api/writing-assistant/shrink", payload).await?;
     if !res.status().is_success() {
        println!("FAILED");
        return Err(anyhow!("Status: {}", res.status()));
    }
    println!("PASSED ({})", res.status());
    Ok(())
}

async fn test_rewrite_success(hc: &Client) -> Result<()> {
    print!("TEST - Rewrite Text: ");
    let payload = json!({ "content": "Hello world.", "style": "formal" });
    let res = hc.do_post("/api/writing-assistant/rewrite", payload).await?;
     if !res.status().is_success() {
        println!("FAILED");
        return Err(anyhow!("Status: {}", res.status()));
    }
    println!("PASSED ({})", res.status());
    Ok(())
}

async fn test_fact_check_success(hc: &Client) -> Result<()> {
    print!("TEST - Fact Check Text: ");
    let payload = json!({ "content": "The sky is green." });
    let res = hc.do_post("/api/writing-assistant/factcheck", payload).await?;
     if !res.status().is_success() {
        println!("FAILED");
        return Err(anyhow!("Status: {}", res.status()));
    }
    println!("PASSED ({})", res.status());
    Ok(())
}

async fn test_spell_check_success(hc: &Client) -> Result<()> {
    print!("TEST - Spell Check Text: ");
    let payload = json!({ "content": "This sentense has spelling mistaks." });
    let res = hc.do_post("/api/writing-assistant/spellcheck", payload).await?;
     if !res.status().is_success() {
        println!("FAILED");
        return Err(anyhow!("Status: {}", res.status()));
    }
    println!("PASSED ({})", res.status());
    Ok(())
}