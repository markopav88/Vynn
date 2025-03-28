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
    let add_permissions_result = test_add_permissions(&hc).await;
    let get_permissions_result = test_get_permissions(&hc).await;
    let update_permission_result = test_update_permission(&hc).await;
    let remove_permissions_result = test_remove_permissions(&hc).await;
    let add_document_result = test_add_document_to_project(&hc).await;
    let get_documents_result = test_get_project_documents(&hc).await;
    let remove_document_result = test_remove_document_from_project(&hc).await;
    let force_delete_project_result = test_force_delete_project(&hc).await;
    let delete_project_result = test_delete_project(&hc).await;
    let reset_db_result = backend::test_reset_db(&hc).await;

    // Print summary
    println!("\n======== TEST RESULTS ========");
    println!("Login as User 1:\t{}", result_to_string(&login_result));
    println!("Create Project:\t\t{}", result_to_string(&create_project_result));
    println!("Get All Projects:\t{}", result_to_string(&get_all_projects_result));
    println!("Get Project:\t\t{}", result_to_string(&get_project_result));
    println!("Update Project:\t\t{}", result_to_string(&update_project_result));
    println!("Add Permissions:\t{}", result_to_string(&add_permissions_result));
    println!("Get Permissions:\t{}", result_to_string(&get_permissions_result));
    println!("Update Permission:\t{}", result_to_string(&update_permission_result));
    println!("Remove Permission:\t{}", result_to_string(&remove_permissions_result));
    println!("Add Document:\t\t{}", result_to_string(&add_document_result));
    println!("Get Documents:\t\t{}", result_to_string(&get_documents_result));
    println!("Remove Document:\t{}", result_to_string(&remove_document_result));
    println!("Force Delete Project:\t{}", result_to_string(&force_delete_project_result));
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

async fn test_add_permissions(hc: &Client) -> Result<()> {
    println!("TEST - Add Project Permissions");
    
    // Add permissions for user 2 on project 1 (default project)
    let add_perm_response = hc
        .do_post(
            "/api/project/1/permissions",
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
    println!("TEST - Get Project Permissions");
    
    // Get permissions for project 1
    let get_perm_response = hc
        .do_get("/api/project/1/permissions")
        .await?;
    
    get_perm_response.print().await?;
    
    if !get_perm_response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Get permissions failed with status: {}",
            get_perm_response.status()
        ));
    }
    
    Ok(())
}

async fn test_update_permission(hc: &Client) -> Result<()> {
    println!("TEST - Update Project Permission");
    
    // Update permissions for user 2 on project 1
    let update_perm_response = hc
        .do_put(
            "/api/project/1/permissions",
            json!({
                "user_id": 2,
                "role": "viewer"
            }),
        )
        .await?;
    
    update_perm_response.print().await?;
    
    if !update_perm_response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Update permission failed with status: {}",
            update_perm_response.status()
        ));
    }
    
    Ok(())
}

async fn test_remove_permissions(hc: &Client) -> Result<()> {
    println!("TEST - Remove Project Permission");
    
    // Remove permissions for user 2 on project 1
    let remove_perm_response = hc
        .do_delete("/api/project/1/permissions/2")
        .await?;
    
    remove_perm_response.print().await?;
    
    if !remove_perm_response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Remove permission failed with status: {}",
            remove_perm_response.status()
        ));
    }
    
    Ok(())
}

async fn test_force_delete_project(hc: &Client) -> Result<()> {
    println!("TEST - Force Delete Project with Documents");
    
    // Use project ID 1 (default project) which should already exist
    let project_id = 1;
    
    // Force delete the project and all its documents
    let force_delete_response = hc
        .do_delete(&format!("/api/project/{}/force", project_id))
        .await?;
    
    force_delete_response.print().await?;
    
    if !force_delete_response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Force delete project failed with status: {}",
            force_delete_response.status()
        ));
    }
    
    Ok(())
}

async fn test_add_document_to_project(hc: &Client) -> Result<()> {
    println!("TEST - Add Document to Project");
    
    // Add document 1 to project 1
    let add_doc_response = hc
        .do_post("/api/project/1/documents/1", json!({}))
        .await?;
    
    add_doc_response.print().await?;
    
    if !add_doc_response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Add document to project failed with status: {}",
            add_doc_response.status()
        ));
    }
    
    Ok(())
}

async fn test_get_project_documents(hc: &Client) -> Result<()> {
    println!("TEST - Get Project Documents");
    
    // Get all documents in project 1
    let get_docs_response = hc
        .do_get("/api/project/1/documents")
        .await?;
    
    get_docs_response.print().await?;
    
    if !get_docs_response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Get project documents failed with status: {}",
            get_docs_response.status()
        ));
    }
    
    Ok(())
}

async fn test_remove_document_from_project(hc: &Client) -> Result<()> {
    println!("TEST - Remove Document from Project");
    
    // Remove document 1 from project 1
    let remove_doc_response = hc
        .do_delete("/api/project/1/documents/1")
        .await?;
    
    remove_doc_response.print().await?;
    
    if !remove_doc_response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Remove document from project failed with status: {}",
            remove_doc_response.status()
        ));
    }
    
    Ok(())
}

