<script lang="ts">
    import { onMount } from 'svelte';
    import { get_all_documents, create_document } from "$lib/ts/document";
    import { get_all_projects, create_project } from "$lib/ts/drive";
    import Navbar from '$lib/components/Navbar.svelte';
    import type { Document } from '$lib/ts/document';
    import type { Project } from '$lib/ts/drive';
    
    let isLoggedIn = true;
    let documents: Document[] = [];
    let projects: Project[] = [];
    let isLoading = true;
    let activeCategory = 'all';
    let showNewProjectModal = false;
    let showNewDocumentModal = false;
    let showProjectDocsModal = false;
    let newProjectName = '';
    let newDocumentName = '';
    let newDocumentContent = '';
    let currentProject: Project | null = null;
    let projectDocuments: Document[] = [];
    let projectDocLoading = false;
    
    onMount(async () => {
        try {
            // Fetch documents and projects in parallel
            const [docsResult, projectsResult] = await Promise.all([
                get_all_documents(),
                get_all_projects()
            ]);
            
            documents = docsResult || [];
            projects = projectsResult || [];
            
            console.log("Documents loaded:", documents);
            console.log("Projects loaded:", projects);
        } catch (error) {
            console.error("Error loading drive content:", error);
        } finally {
            isLoading = false;
        }
    });
    
    function setActiveCategory(category: string) {
        activeCategory = category;
    }
    
    // Handle new project creation
    async function handleCreateProject() {
        if (!newProjectName.trim()) {
            alert('Project name cannot be empty');
            return;
        }
        
        try {
            const project = await create_project(newProjectName);
            if (project) {
                // Add the new project to the list
                projects = [...projects, project];
                // Clear the form and close the modal
                newProjectName = '';
                showNewProjectModal = false;
            } else {
                alert('Failed to create project');
            }
        } catch (error) {
            console.error('Error creating project:', error);
            alert('An error occurred while creating the project');
        }
    }
    
    // Handle new document creation
    async function handleCreateDocument() {
        if (!newDocumentName.trim()) {
            alert('Document name cannot be empty');
            return;
        }
        
        try {
            // Create a new document with the name and content
            const document = await create_document(newDocumentName, newDocumentContent || '');
            
            if (document) {
                // Refresh the documents list
                const refreshedDocs = await get_all_documents();
                documents = refreshedDocs || [];
                
                // Clear the form and close the modal
                newDocumentName = '';
                newDocumentContent = '';
                showNewDocumentModal = false;
                
                // Navigate to the document
                window.location.href = `/document/${document.id}`;
            } else {
                alert('Failed to create document');
            }
        } catch (error) {
            console.error('Error creating document:', error);
            alert('An error occurred while creating the document');
        }
    }
    
    // Handle project click
    async function handleProjectClick(project: Project) {
        currentProject = project;
        projectDocLoading = true;
        showProjectDocsModal = true;
        
        try {
            // Fetch documents for this project
            const response = await fetch(`http://localhost:3001/api/project/${project.id}/documents`, {
                credentials: 'include'
            });
            
            if (response.ok) {
                projectDocuments = await response.json();
            } else {
                console.error('Failed to fetch project documents');
                projectDocuments = [];
            }
        } catch (error) {
            console.error('Error fetching project documents:', error);
            projectDocuments = [];
        } finally {
            projectDocLoading = false;
        }
    }
    
    // Handle document click
    function handleDocumentClick(document: Document) {
        // Navigate to document editor
        window.location.href = `/document/${document.id}`;
    }
    
    // Create new document within a project
    async function createDocInProject() {
        if (!currentProject) return;
        
        // Close the current modal
        showProjectDocsModal = false;
        
        // Open the new document modal
        newDocumentName = '';
        newDocumentContent = '';
        showNewDocumentModal = true;
    }
</script>

<div class="bg-black min-vh-100 d-flex flex-column">
    <Navbar {isLoggedIn} />
    
    <div class="container-fluid flex-grow-1 d-flex">
        <div class="row flex-grow-1 w-100 m-0">
            <!-- Left Sidebar -->
            <div class="col-md-2 bg-dark p-0 border-end border-dark min-vh-100">
                <div class="d-flex flex-column h-100 sticky-top">
                    
                    <!-- Navigation Categories -->
                    <div class="p-2">
                        <ul class="nav flex-column">
                            <li class="nav-item">
                                <button class="nav-link text-white {activeCategory === 'all' ? 'active bg-black' : ''}" 
                                       on:click={() => setActiveCategory('all')}>
                                    <i class="bi bi-grid me-2"></i> All Items
                                </button>
                            </li>
                            <li class="nav-item">
                                <button class="nav-link text-white {activeCategory === 'recent' ? 'active bg-black' : ''}" 
                                       on:click={() => setActiveCategory('recent')}>
                                    <i class="bi bi-clock-history me-2"></i> Recent
                                </button>
                            </li>
                            <li class="nav-item">
                                <button class="nav-link text-white {activeCategory === 'shared' ? 'active bg-black' : ''}" 
                                       on:click={() => setActiveCategory('shared')}>
                                    <i class="bi bi-people me-2"></i> Shared with me
                                </button>
                            </li>
                            <li class="nav-item">
                                <button class="nav-link text-white {activeCategory === 'starred' ? 'active bg-black' : ''}" 
                                       on:click={() => setActiveCategory('starred')}>
                                    <i class="bi bi-star me-2"></i> Starred
                                </button>
                            </li>
                            <li class="nav-item">
                                <button class="nav-link text-white {activeCategory === 'trash' ? 'active bg-black' : ''}" 
                                       on:click={() => setActiveCategory('trash')}>
                                    <i class="bi bi-trash me-2"></i> Trash
                                </button>
                            </li>
                            <li class="d-flex justify-content-between nav-item mt-2 pt-2 border-top border-dark">
                                <button class="nav-link text-white w-100 d-flex justify-content-between" style="cursor: default;">
                                    <span><i class="bi bi-hdd me-2"></i> Storage</span>
                                    <span class="text-white-50">25%</span>
                                </button>
                            </li>
                            <li class="px-2">
                                <div class="progress mb-2 mt-1" style="height: 6px;">
                                    <div class="progress-bar bg-green" role="progressbar" style="width: 25%;" aria-valuenow="25" aria-valuemin="0" aria-valuemax="100"></div>
                                </div>
                                <small class="text-white-50 ps-1">2.5 GB of 10 GB used</small>
                            </li>
                        </ul>
                    </div>
                    
                    <!-- Create New Buttons -->
                    <div class="mt-4 p-2">
                        <button class="btn btn-green w-100 mb-2" on:click={() => showNewProjectModal = true}>
                            <i class="bi bi-folder-plus me-2"></i> New Project
                        </button>
                        <button class="btn btn-outline-light w-100" on:click={() => showNewDocumentModal = true}>
                            <i class="bi bi-file-earmark-plus me-2"></i> New Document
                        </button>
                    </div>
                </div>
            </div>
            
            <!-- Main Content Area -->
            <div class="col-md-10 bg-black p-4">
                <h1 class="mb-4">My Drive</h1>
                
                {#if isLoading}
                    <div class="d-flex justify-content-center my-5">
                        <div class="spinner-border text-green" role="status">
                            <span class="visually-hidden">Loading...</span>
                        </div>
                    </div>
                {:else}
                    <!-- Unified Items Section -->
                    <div>
                        <div class="d-flex justify-content-between align-items-center mb-4">
                            <h2>All Items</h2>
                            <div>
                                <button class="btn btn-sm btn-outline-green me-2" on:click={() => showNewProjectModal = true}>
                                    <i class="bi bi-folder-plus me-1"></i> New Project
                                </button>
                                <button class="btn btn-sm btn-outline-light" on:click={() => showNewDocumentModal = true}>
                                    <i class="bi bi-file-earmark-plus me-1"></i> New Document
                                </button>
                            </div>
                        </div>
                        
                        {#if projects.length === 0 && documents.length === 0}
                            <div class="text-white-50 p-5 text-center border border-dark rounded">
                                <i class="bi bi-inbox display-4 d-block mb-3"></i>
                                <p>No items found</p>
                                <div class="mt-3">
                                    <button class="btn btn-sm btn-outline-green me-2" on:click={() => showNewProjectModal = true}>Create Project</button>
                                    <button class="btn btn-sm btn-outline-light" on:click={() => showNewDocumentModal = true}>Create Document</button>
                                </div>
                            </div>
                        {:else}
                            <div class="row row-cols-1 row-cols-md-4 g-4">
                                <!-- Projects First -->
                                {#each projects as project}
                                    <div class="col">
                                        <div class="card bg-dark border-0 h-100 item-card project-card" on:click={() => handleProjectClick(project)}>
                                            <div class="card-body p-3">
                                                <div class="d-flex align-items-center mb-2">
                                                    <i class="bi bi-folder-fill text-green fs-4 me-2"></i>
                                                    <h5 class="card-title mb-0 text-truncate">{project.name}</h5>
                                                </div>
                                                <p class="card-text text-white-50 small mb-1">Project</p>
                                                <p class="card-text text-white-50 small">
                                                    Updated: {new Date(project.updated_at).toLocaleDateString()}
                                                </p>
                                            </div>
                                        </div>
                                    </div>
                                {/each}
                                
                                <!-- Documents After Projects -->
                                {#each documents as document}
                                    <div class="col">
                                        <div class="card bg-dark border-0 h-100 item-card document-card" on:click={() => handleDocumentClick(document)}>
                                            <div class="card-body p-3">
                                                <div class="d-flex align-items-center mb-2">
                                                    <i class="bi bi-file-earmark-text text-white-50 fs-4 me-2"></i>
                                                    <h5 class="card-title mb-0 text-truncate">{document.name}</h5>
                                                </div>
                                                <p class="card-text text-white-50 small mb-1">Document</p>
                                                <p class="card-text text-white-50 small">
                                                    Updated: {new Date(document.updated_at).toLocaleDateString()}
                                                </p>
                                            </div>
                                        </div>
                                    </div>
                                {/each}
                            </div>
                        {/if}
                    </div>
                {/if}
            </div>
        </div>
    </div>
</div>

<!-- New Project Modal -->
{#if showNewProjectModal}
<div class="modal fade show d-block" tabindex="-1" role="dialog" aria-hidden="true">
    <div class="modal-dialog modal-dialog-centered">
        <div class="modal-content bg-dark text-white">
            <div class="modal-header border-secondary">
                <h5 class="modal-title">Create New Project</h5>
                <button type="button" class="btn-close btn-close-white" aria-label="Close" on:click={() => showNewProjectModal = false}></button>
            </div>
            <div class="modal-body">
                <form on:submit|preventDefault={handleCreateProject}>
                    <div class="mb-3">
                        <label for="projectName" class="form-label">Project Name</label>
                        <input type="text" class="form-control bg-dark text-white border-secondary" 
                               id="projectName" bind:value={newProjectName} 
                               placeholder="Enter project name" required>
                    </div>
                </form>
            </div>
            <div class="modal-footer border-secondary">
                <button type="button" class="btn btn-outline-light" on:click={() => showNewProjectModal = false}>Cancel</button>
                <button type="button" class="btn btn-green" on:click={handleCreateProject}>Create Project</button>
            </div>
        </div>
    </div>
</div>
<div class="modal-backdrop fade show"></div>
{/if}

<!-- New Document Modal -->
{#if showNewDocumentModal}
<div class="modal fade show d-block" tabindex="-1" role="dialog" aria-hidden="true">
    <div class="modal-dialog modal-dialog-centered">
        <div class="modal-content bg-dark text-white">
            <div class="modal-header border-secondary">
                <h5 class="modal-title">Create New Document</h5>
                <button type="button" class="btn-close btn-close-white" aria-label="Close" on:click={() => showNewDocumentModal = false}></button>
            </div>
            <div class="modal-body">
                <form on:submit|preventDefault={handleCreateDocument}>
                    <div class="mb-3">
                        <label for="documentName" class="form-label">Document Name</label>
                        <input type="text" class="form-control bg-dark text-white border-secondary" 
                               id="documentName" bind:value={newDocumentName} 
                               placeholder="Enter document name" required>
                    </div>
                </form>
            </div>
            <div class="modal-footer border-secondary">
                <button type="button" class="btn btn-outline-light" on:click={() => showNewDocumentModal = false}>Cancel</button>
                <button type="button" class="btn btn-green" on:click={handleCreateDocument}>Create Document</button>
            </div>
        </div>
    </div>
</div>
<div class="modal-backdrop fade show"></div>
{/if}

<!-- Project Documents Modal -->
{#if showProjectDocsModal && currentProject}
<div class="modal fade show d-block" tabindex="-1" role="dialog" aria-hidden="true">
    <div class="modal-dialog modal-dialog-centered modal-lg">
        <div class="modal-content bg-dark text-white">
            <div class="modal-header border-secondary">
                <h5 class="modal-title">
                    <i class="bi bi-folder-fill text-green me-2"></i>
                    {currentProject.name}
                </h5>
                <button type="button" class="btn-close btn-close-white" aria-label="Close" on:click={() => showProjectDocsModal = false}></button>
            </div>
            <div class="modal-body">
                {#if projectDocLoading}
                    <div class="d-flex justify-content-center my-4">
                        <div class="spinner-border text-green" role="status">
                            <span class="visually-hidden">Loading...</span>
                        </div>
                    </div>
                {:else if projectDocuments.length === 0}
                    <div class="text-center py-4">
                        <i class="bi bi-file-earmark-text display-4 text-white-50 mb-3"></i>
                        <p>No documents in this project</p>
                        <button class="btn btn-outline-light mt-2" on:click={createDocInProject}>
                            <i class="bi bi-file-earmark-plus me-2"></i>
                            Create Document
                        </button>
                    </div>
                {:else}
                    <div class="row row-cols-1 row-cols-md-3 g-4">
                        {#each projectDocuments as document}
                            <div class="col">
                                <div class="card bg-black border-0 h-100 document-card" on:click={() => handleDocumentClick(document)}>
                                    <div class="card-body p-3">
                                        <div class="d-flex align-items-center mb-2">
                                            <i class="bi bi-file-earmark-text text-white-50 fs-4 me-2"></i>
                                            <h5 class="card-title mb-0 text-truncate">{document.name}</h5>
                                        </div>
                                        <p class="card-text text-white-50 small mb-1">
                                            Last updated: {new Date(document.updated_at).toLocaleDateString()}
                                        </p>
                                    </div>
                                </div>
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>
            <div class="modal-footer border-secondary">
                <button type="button" class="btn btn-outline-light" on:click={() => showProjectDocsModal = false}>Close</button>
                <button type="button" class="btn btn-green" on:click={createDocInProject}>
                    <i class="bi bi-file-earmark-plus me-2"></i>
                    New Document
                </button>
            </div>
        </div>
    </div>
</div>
<div class="modal-backdrop fade show"></div>
{/if}

<style>
    /* Custom styles for the drive page */
    .nav-link {
        border-radius: 5px;
        margin-bottom: 5px;
        transition: all 0.2s;
    }
    
    .nav-link:hover {
        background-color: rgba(255, 255, 255, 0.1);
    }
    
    .nav-link.active {
        font-weight: 500;
        color: var(--color-primary) !important;
    }
    
    .item-card {
        transition: all 0.2s;
        cursor: pointer;
        background: linear-gradient(145deg, #0a0a0a, #1a1a1a);
        border-radius: 8px;
        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    }
    
    .item-card:hover {
        transform: translateY(-3px);
        box-shadow: 0 6px 12px rgba(0, 0, 0, 0.15);
    }
    
    .project-card:hover {
        border-left: 3px solid var(--color-primary);
    }
    
    .document-card:hover {
        border-left: 3px solid #6c757d;
    }
    
    /* Green glow for project cards */
    .project-card::before {
        content: '';
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        border-radius: 8px;
        padding: 1px;
        background: linear-gradient(145deg, rgba(16, 185, 129, 0.2), transparent);
        -webkit-mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
        mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
        -webkit-mask-composite: xor;
        mask-composite: exclude;
        pointer-events: none;
    }
</style> 