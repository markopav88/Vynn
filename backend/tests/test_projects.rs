use anyhow::Result;
use backend::result_to_string;
use httpc_test::Client;
use serde_json::json;

#[tokio::test]
async fn test_projects() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:3001")?;

    println!("\n===== RUNNING PROJECT API TESTS =====\n");

    // Run all tests and collect results
    let login_result = test_login(&hc).await;
    let create_project_result = test_create_project(&hc).await;
    let get_all_projects_result = test_get_all_projects(&hc).await;
    let get_project_result = test_get_project(&hc).await;
    let update_project_result = test_update_project(&hc).await;
    let delete_project_result = test_delete_project(&hc).await;
    let reset_db_result = backend::test_reset_db(&hc).await;

    // Print summary
    println!("\n======== TEST RESULTS ========");
    println!("Login as User 1:\t{}", result_to_string(&login_result));
    println!("Create Project:\t\t{}", result_to_string(&create_project_result));
    println!("Get All Projects:\t{}", result_to_string(&get_all_projects_result));
    println!("Get Project:\t\t{}", result_to_string(&get_project_result));
    println!("Update Project:\t\t{}", result_to_string(&update_project_result));
    println!("Delete Project:\t\t{}", result_to_string(&delete_project_result));
    println!("Reset Database:\t\t{}", result_to_string(&reset_db_result));
    println!("==============================\n");

    Ok(())
}

async fn test_login(hc: &Client) -> Result<()> {
    println!("TEST - Login for Project Tests");
    let response = hc
        .do_post(
            "/api/login",
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


async fn test_create_project(hc: &Client) -> Result<()> {
    println!("TEST - Create Project");
    let response = hc
        .do_post(
            "/api/project",
            json!({
                "_name": "Test Project"
            }),
        )
        .await?;
    
    response.print().await?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Project creation failed with status: {}",
            response.status()
        ));
    }
    
    Ok(())
}

async fn test_get_all_projects(hc: &Client) -> Result<()> {
    println!("TEST - Get All Projects");

    // now get all projects including the one we just made
    let response = hc.do_get("/api/project").await?;
    
    response.print().await?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Get all projects failed with status: {}",
            response.status()
        ));
    }
    
    Ok(())
}

async fn test_get_project(hc: &Client) -> Result<()> {
    println!("TEST - Get Project");
    
    // Now get the project we just made
    let get_response = hc.do_get(&format!("/api/project/2")).await?;
    
    get_response.print().await?;
    
    if !get_response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Get project failed with status: {}",
            get_response.status()
        ));
    }
    
    Ok(())
}

async fn test_update_project(hc: &Client) -> Result<()> {
    println!("TEST - Update Project");

    // Update the created project
    let update_response = hc
        .do_put(
            &format!("/api/project/2"),
            json!({
                "_name": "Updated Project Name"
            }),
        )
        .await?;
    
    update_response.print().await?;
    
    if !update_response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Update project failed with status: {}",
            update_response.status()
        ));
    }
    
    Ok(())
}

async fn test_delete_project(hc: &Client) -> Result<()> {
    println!("TEST - Delete Project");

    // Now delete the created project
    let delete_response = hc
        .do_delete(&format!("/api/project/2"))
        .await?;
    
    delete_response.print().await?;
    
    if !delete_response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Delete project failed with status: {}",
            delete_response.status()
        ));
    }
    
    Ok(())
}

