#![allow(unused)]

use anyhow::{anyhow, Result};
use backend::result_to_string;
use chrono::Utc;
use httpc_test::Client;
use serde_json::json;

#[tokio::test]
async fn test_writing_assistant() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:3001")?;

    println!("\n===== RUNNING WRITING ASSISTANT API TESTS =====\n");

    // Run all tests and collect results
    let login_result = test_good_login(&hc).await;
    let create_session = test_create_writing_session_success(&hc).await;
    let get_all_sessions = test_get_all_writing_sessions_success(&hc).await;
    let get_session = test_get_writing_session_success(&hc).await;
    let send_message = test_send_writing_message_success(&hc).await;
    let check_grammar = test_check_grammar_success(&hc).await;
    let spell_check = test_spell_check_success(&hc).await;
    let summarize = test_summarize_success(&hc).await;
    let rephrase = test_rephrase_success(&hc).await;
    let expand = test_expand_success(&hc).await;
    let shrink = test_shrink_success(&hc).await;
    let rewrite = test_rewrite_success(&hc).await;
    let fact_check = test_fact_check_success(&hc).await;
    let apply_suggestion = test_apply_suggestion_success(&hc).await;
    let decide_proactive = test_decide_proactive_diff_success(&hc).await;
    let sanitize_text = test_sanitize_text_success(&hc).await;
    let delete_session = test_delete_writing_session_success(&hc).await;
    let reset_db = backend::test_reset_db(&hc).await;

    // Print summary
    println!("\n======== TEST RESULTS ========");
    println!("Login as User 1\t\t{}", result_to_string(&login_result));
    println!("Create Session\t\t{}", result_to_string(&create_session));
    println!("Get All Sessions\t\t{}", result_to_string(&get_all_sessions));
    println!("Get Session\t\t{}", result_to_string(&get_session));
    println!("Send Message\t\t{}", result_to_string(&send_message));
    println!("Check Grammar\t\t{}", result_to_string(&check_grammar));
    println!("Spell Check\t\t{}", result_to_string(&spell_check));
    println!("Summarize\t\t{}", result_to_string(&summarize));
    println!("Rephrase\t\t{}", result_to_string(&rephrase));
    println!("Expand\t\t\t{}", result_to_string(&expand));
    println!("Shrink\t\t\t{}", result_to_string(&shrink));
    println!("Rewrite\t\t\t{}", result_to_string(&rewrite));
    println!("Fact Check\t\t{}", result_to_string(&fact_check));
    println!("Apply Suggestion\t\t{}", result_to_string(&apply_suggestion));
    println!("Decide Proactive\t\t{}", result_to_string(&decide_proactive));
    println!("Sanitize Text\t\t{}", result_to_string(&sanitize_text));
    println!("Delete Session\t\t{}", result_to_string(&delete_session));
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

async fn test_create_writing_session_success(hc: &Client) -> Result<()> {
    println!("TEST - Create Writing Session");

    let response = hc
        .do_post(
            "/api/writing-assistant",
            json!({
                "title": "Test Writing Session",
                "document_id": null
            }),
        )
        .await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Create writing session failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_get_all_writing_sessions_success(hc: &Client) -> Result<()> {
    println!("TEST - Get All Writing Sessions");

    let response = hc.do_get("/api/writing-assistant").await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Get all writing sessions failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_get_writing_session_success(hc: &Client) -> Result<()> {
    println!("TEST - Get Writing Session");

    // Use session ID 1 for testing
    let response = hc.do_get("/api/writing-assistant/1").await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Get writing session failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_send_writing_message_success(hc: &Client) -> Result<()> {
    println!("TEST - Send Writing Message");

    let response = hc
        .do_post(
            "/api/writing-assistant/1/message",
            json!({
                "content": "Hello, can you help me with my writing?"
            }),
        )
        .await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Send writing message failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_check_grammar_success(hc: &Client) -> Result<()> {
    println!("TEST - Check Grammar");

    let response = hc
        .do_post(
            "/api/writing-assistant/grammer",
            json!({
                "content": "This sentence have bad grammar."
            }),
        )
        .await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Grammar check failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_spell_check_success(hc: &Client) -> Result<()> {
    println!("TEST - Spell Check");

    let response = hc
        .do_post(
            "/api/writing-assistant/spellcheck",
            json!({
                "content": "This sentense has a speling mistake."
            }),
        )
        .await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Spell check failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_summarize_success(hc: &Client) -> Result<()> {
    println!("TEST - Summarize Text");

    let response = hc
        .do_post(
            "/api/writing-assistant/summarize",
            json!({
                "content": "This is a long text that needs to be summarized. It contains multiple sentences and ideas that could be condensed into a shorter form."
            }),
        )
        .await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Summarize failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_rephrase_success(hc: &Client) -> Result<()> {
    println!("TEST - Rephrase Text");

    let response = hc
        .do_post(
            "/api/writing-assistant/rephrase",
            json!({
                "content": "This sentence needs to be rephrased."
            }),
        )
        .await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Rephrase failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_expand_success(hc: &Client) -> Result<()> {
    println!("TEST - Expand Text");

    let response = hc
        .do_post(
            "/api/writing-assistant/expand",
            json!({
                "content": "AI is useful."
            }),
        )
        .await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Expand failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_shrink_success(hc: &Client) -> Result<()> {
    println!("TEST - Shrink Text");

    let response = hc
        .do_post(
            "/api/writing-assistant/shrink",
            json!({
                "content": "This is a very long and verbose text that contains many unnecessary words and could be made much more concise."
            }),
        )
        .await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Shrink failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_rewrite_success(hc: &Client) -> Result<()> {
    println!("TEST - Rewrite Text");

    let response = hc
        .do_post(
            "/api/writing-assistant/rewrite",
            json!({
                "content": "This text needs to be rewritten.",
                "style": "professional"
            }),
        )
        .await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Rewrite failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_fact_check_success(hc: &Client) -> Result<()> {
    println!("TEST - Fact Check");

    let response = hc
        .do_post(
            "/api/writing-assistant/factcheck",
            json!({
                "content": "The Earth is flat and the sun revolves around it."
            }),
        )
        .await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Fact check failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_apply_suggestion_success(hc: &Client) -> Result<()> {
    println!("TEST - Apply Suggestion");

    let response = hc
        .do_post(
            "/api/writing-assistant/1/apply-suggestion",
            json!({
                "suggestion_content": "Fix the grammar in this document.",
                "current_document_id": 1
            }),
        )
        .await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Apply suggestion failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_decide_proactive_diff_success(hc: &Client) -> Result<()> {
    println!("TEST - Decide Proactive Diff");

    let response = hc
        .do_post(
            "/api/writing-assistant/decide-proactive-diff",
            json!({
                "ai_response_content": "Here's a suggestion to improve your text.",
                "context": {
                    "current_mode": "writing",
                    "last_user_action": "edit",
                    "time_since_last_suggestion": 300
                },
                "document_content_snippet": "This is the current document content."
            }),
        )
        .await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Decide proactive diff failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_sanitize_text_success(hc: &Client) -> Result<()> {
    println!("TEST - Sanitize Text");

    let response = hc
        .do_post(
            "/api/writing-assistant/sanitize-text",
            json!({
                "text_to_sanitize": "<p>This text has HTML</p> and **markdown** formatting."
            }),
        )
        .await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Sanitize text failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}

async fn test_delete_writing_session_success(hc: &Client) -> Result<()> {
    println!("TEST - Delete Writing Session");

    let response = hc.do_delete("/api/writing-assistant/1").await?;
    response.print().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Delete writing session failed with status: {}",
            response.status()
        ));
    }

    Ok(())
}