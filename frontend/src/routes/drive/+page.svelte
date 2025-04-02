<script lang="ts">
    import { onMount } from 'svelte';
    import { get_all_documents, create_document, toggle_star_document, trash_document, restore_document, get_starred_documents, get_trashed_documents } from "$lib/ts/document";
    import { get_all_projects, create_project, toggle_star_project, trash_project, restore_project, get_starred_projects, get_trashed_projects } from "$lib/ts/drive";
    import { add_document_to_project, get_project_documents } from "$lib/ts/project";
    import Navbar from '$lib/components/Navbar.svelte';
    import type { Document } from '$lib/ts/document';
    import type { Project } from '$lib/ts/drive';
    
    let isLoggedIn = true;
    let documents: Document[] = [];
    let projects: Project[] = [];
    let starredDocuments: Document[] = [];
    let starredProjects: Project[] = [];
    let trashedDocuments: Document[] = [];
    let trashedProjects: Project[] = [];
    let isLoading = true;
    let activeCategory = 'all';
    let showNewProjectModal = false;
    let showNewDocumentModal = false;
    let showProjectDocsModal = false;
    let newProjectName = '';
    let newDocumentName = '';
    let newDocumentContent = '';
    let newDocumentProjectId: string | null = null;
    let currentProject: Project | null = null;
    let projectDocuments: Document[] = [];
    let projectDocLoading = false;
    let draggedDocument: Document | null = null;
    
    onMount(async () => {
        try {
            // Fetch all data in parallel
            const [docsResult, projectsResult, starredDocsResult, starredProjsResult, trashedDocsResult, trashedProjsResult] = await Promise.all([
                get_all_documents(),
                get_all_projects(),
                get_starred_documents(),
                get_starred_projects(),
                get_trashed_documents(),
                get_trashed_projects()
            ]);
            
            documents = docsResult || [];
            projects = projectsResult || [];
            starredDocuments = starredDocsResult || [];
            starredProjects = starredProjsResult || [];
            trashedDocuments = trashedDocsResult || [];
            trashedProjects = trashedProjsResult || [];
            
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
                
                // Show the project documents modal immediately after creation
                handleProjectClick(project);
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
                // If we have a project ID, associate the document with the project
                if (newDocumentProjectId) {
                    await add_document_to_project(parseInt(newDocumentProjectId), document.id);
                }
                
                // Refresh the documents list
                const refreshedDocs = await get_all_documents();
                documents = refreshedDocs || [];
                
                // Clear the form and close the modal
                newDocumentName = '';
                newDocumentContent = '';
                newDocumentProjectId = null;
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
            const projectDocs = await get_project_documents(parseInt(project.id));
            
            if (projectDocs) {
                projectDocuments = projectDocs;
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
        if (!document.is_trashed) {
            // Navigate to document editor
            window.location.href = `/document/${document.id}`;
        }
    }
    
    // Create new document within a project
    async function createDocInProject() {
        if (!currentProject) return;
        
        // Close the current modal
        showProjectDocsModal = false;
        
        // Open the new document modal with project context
        newDocumentName = '';
        newDocumentContent = '';
        newDocumentProjectId = currentProject.id;
        showNewDocumentModal = true;
    }
    
    // Toggle star status for a document
    async function handleToggleStarDocument(event: Event, document: Document) {
        event.stopPropagation(); // Prevent document click
        
        const success = await toggle_star_document(document);
        if (success) {
            // Toggle the starred status locally
            document.is_starred = !document.is_starred;
            
            // Update the starred documents list
            if (document.is_starred) {
                starredDocuments = [...starredDocuments, document];
            } else {
                starredDocuments = starredDocuments.filter(d => d.id !== document.id);
            }
        }
    }
    
    // Toggle star status for a project
    async function handleToggleStarProject(event: Event, project: Project) {
        event.stopPropagation(); // Prevent project click
        
        const success = await toggle_star_project(parseInt(project.id));
        if (success) {
            // Toggle the starred status locally
            project.is_starred = !project.is_starred;
            
            // Update the starred projects list
            if (project.is_starred) {
                starredProjects = [...starredProjects, project];
            } else {
                starredProjects = starredProjects.filter(p => p.id !== project.id);
            }
        }
    }
    
    // Move document to trash
    async function handleTrashDocument(event: Event, document: Document) {
        event.stopPropagation(); // Prevent document click
        
        const success = await trash_document(document);
        if (success) {
            // Update the document's trashed status locally
            document.is_trashed = true;
            
            // Update the trashed documents list
            trashedDocuments = [...trashedDocuments, document];
            
            // Remove from starred documents if it was starred
            if (document.is_starred) {
                starredDocuments = starredDocuments.filter(d => d.id !== document.id);
            }
        }
    }
    
    // Move project to trash
    async function handleTrashProject(event: Event, project: Project) {
        event.stopPropagation(); // Prevent project click
        
        const success = await trash_project(parseInt(project.id));
        if (success) {
            // Update the project's trashed status locally
            project.is_trashed = true;
            
            // Update the trashed projects list
            trashedProjects = [...trashedProjects, project];
            
            // Remove from starred projects if it was starred
            if (project.is_starred) {
                starredProjects = starredProjects.filter(p => p.id !== project.id);
            }
        }
    }
    
    // Restore document from trash
    async function handleRestoreDocument(event: Event, document: Document) {
        event.stopPropagation(); // Prevent document click
        
        const success = await restore_document(document);
        if (success) {
            // Update the document's trashed status locally
            document.is_trashed = false;
            
            // Remove from trashed documents list
            trashedDocuments = trashedDocuments.filter(d => d.id !== document.id);
        }
    }
    
    // Restore project from trash
    async function handleRestoreProject(event: Event, project: Project) {
        event.stopPropagation(); // Prevent project click
        
        const success = await restore_project(parseInt(project.id));
        if (success) {
            // Update the project's trashed status locally
            project.is_trashed = false;
            
            // Remove from trashed projects list
            trashedProjects = trashedProjects.filter(p => p.id !== project.id);
        }
    }
    
    // Drag and drop functionality
    function handleDragStart(event: DragEvent, document: Document) {
        draggedDocument = document;
        if (event.dataTransfer) {
            event.dataTransfer.setData('text/plain', document.id.toString());
            event.dataTransfer.effectAllowed = 'move';
        }
    }
    
    function handleDragOver(event: DragEvent, project: Project) {
        event.preventDefault();
        if (event.currentTarget) {
            (event.currentTarget as HTMLElement).classList.add('drag-over');
        }
    }
    
    function handleDragLeave(event: DragEvent) {
        if (event.currentTarget) {
            (event.currentTarget as HTMLElement).classList.remove('drag-over');
        }
    }
    
    async function handleDrop(event: DragEvent, project: Project) {
        event.preventDefault();
        if (event.currentTarget) {
            (event.currentTarget as HTMLElement).classList.remove('drag-over');
        }
        
        if (draggedDocument) {
            // Add document to project
            const success = await add_document_to_project(parseInt(project.id), draggedDocument.id);
            
            if (success) {
                // Show success message or update UI
                alert(`Document "${draggedDocument.name}" added to project "${project.name}"`);
            }
            
            draggedDocument = null;
        }
    }
    
    // Computed properties for filtered views
    $: filteredDocuments = activeCategory === 'all' 
        ? documents.filter(d => !d.is_trashed)
        : activeCategory === 'starred' 
            ? starredDocuments.filter(d => !d.is_trashed)
            : activeCategory === 'trash'
                ? trashedDocuments
                : activeCategory === 'recent'
                    ? [...documents].filter(d => !d.is_trashed).sort((a, b) => 
                        new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
                    ).slice(0, 10)
                    : [];
    
    $: filteredProjects = activeCategory === 'all'
        ? projects.filter(p => !p.is_trashed)
        : activeCategory === 'starred'
            ? starredProjects.filter(p => !p.is_trashed)
            : activeCategory === 'trash'
                ? trashedProjects
                : activeCategory === 'recent'
                    ? [...projects].filter(p => !p.is_trashed).sort((a, b) => 
                        new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
                    ).slice(0, 10)
                    : [];
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
                                {#each filteredProjects as project}
                                    <div class="col">
                                        <div class="card bg-dark border-0 h-100 item-card project-card position-relative" 
                                            on:click={() => handleProjectClick(project)}
                                            on:dragover={(e) => handleDragOver(e, project)}
                                            on:dragleave={handleDragLeave}
                                            on:drop={(e) => handleDrop(e, project)}>
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
                                            
                                            <!-- Action icons for projects -->
                                            <div class="card-actions">
                                                {#if activeCategory === 'trash'}
                                                    <button class="action-icon restore-icon" on:click={(e) => handleRestoreProject(e, project)} title="Restore">
                                                        <i class="bi bi-arrow-counterclockwise"></i>
                                                    </button>
                                                {:else}
                                                    <button class="action-icon star-icon" on:click={(e) => handleToggleStarProject(e, project)} title="Star">
                                                        <i class="bi {project.is_starred ? 'bi-star-fill text-warning' : 'bi-star'}"></i>
                                                    </button>
                                                    <button class="action-icon trash-icon" on:click={(e) => handleTrashProject(e, project)} title="Trash">
                                                        <i class="bi bi-trash"></i>
                                                    </button>
                                                {/if}
                                            </div>
                                        </div>
                                    </div>
                                {/each}
                                
                                <!-- Documents After Projects -->
                                {#each filteredDocuments as document}
                                    <div class="col">
                                        <div class="card bg-dark border-0 h-100 item-card document-card position-relative" 
                                            on:click={() => handleDocumentClick(document)}
                                            draggable={!document.is_trashed}
                                            on:dragstart={(e) => handleDragStart(e, document)}>
                                            <div class="card-body p-3">
                                                <div class="d-flex align-items-center mb-2">
                                                    <i class="bi bi-file-earmark-text text-green fs-4 me-2"></i>
                                                    <h5 class="card-title mb-0 text-truncate text-green">{document.name}</h5>
                                                </div>
                                                <p class="card-text text-white-50 small mb-1">Document</p>
                                                <p class="card-text text-white-50 small">
                                                    Updated: {new Date(document.updated_at).toLocaleDateString()}
                                                </p>
                                            </div>
                                            
                                            <!-- Action icons for documents -->
                                            <div class="card-actions">
                                                {#if activeCategory === 'trash'}
                                                    <button class="action-icon restore-icon" on:click={(e) => handleRestoreDocument(e, document)} title="Restore">
                                                        <i class="bi bi-arrow-counterclockwise"></i>
                                                    </button>
                                                {:else}
                                                    <button class="action-icon star-icon" on:click={(e) => handleToggleStarDocument(e, document)} title="Star">
                                                        <i class="bi {document.is_starred ? 'bi-star-fill text-warning' : 'bi-star'}"></i>
                                                    </button>
                                                    <button class="action-icon trash-icon" on:click={(e) => handleTrashDocument(e, document)} title="Trash">
                                                        <i class="bi bi-trash"></i>
                                                    </button>
                                                {/if}
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
                                <div 
                                    class="card bg-black border-0 h-100 document-card" 
                                    role="button"
                                    tabindex="0"
                                    on:click={() => handleDocumentClick(document)}
                                    on:keydown={(e) => e.key === 'Enter' && handleDocumentClick(document)}
                                >
                                    <div class="card-body p-3">
                                        <div class="d-flex align-items-center mb-2">
                                            <i class="bi bi-file-earmark-text text-green fs-4 me-2"></i>
                                            <h5 class="card-title mb-0 text-truncate text-green">{document.name}</h5>
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
    
    /* Card actions styles */
    .card-actions {
        position: absolute;
        top: 10px;
        right: 10px;
        display: flex;
        gap: 8px;
        opacity: 0;
        transition: opacity 0.2s ease;
    }
    
    .item-card:hover .card-actions {
        opacity: 1;
    }
    
    .action-icon {
        width: 30px;
        height: 30px;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        background-color: rgba(0, 0, 0, 0.5);
        border: 1px solid rgba(255, 255, 255, 0.1);
        color: #ffffff;
        cursor: pointer;
        transition: all 0.2s ease;
    }
    
    .action-icon:hover {
        transform: scale(1.1);
    }
    
    .star-icon:hover {
        color: #ffc107;
        border-color: #ffc107;
    }
    
    .trash-icon:hover {
        color: #dc3545;
        border-color: #dc3545;
    }
    
    .restore-icon:hover {
        color: #10B981;
        border-color: #10B981;
    }
    
    /* Green text for document titles */
    .text-green {
        color: #10B981 !important;
    }
    
    /* Drag and drop styles */
    .drag-over {
        border: 2px dashed #10B981 !important;
        background-color: rgba(16, 185, 129, 0.1) !important;
    }
</style> 