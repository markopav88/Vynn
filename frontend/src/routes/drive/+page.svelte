<script lang="ts">
	import { onMount } from 'svelte';
    import { get_all_documents, create_document, toggle_star_document, trash_document, restore_document, get_starred_documents, get_trashed_documents, type Document } from "$lib/ts/document";
    import { get_all_projects, toggle_star_project, trash_project, restore_project, get_starred_projects, get_trashed_projects, create_project } from "$lib/ts/drive";
    import { add_document_to_project, get_project_documents } from "$lib/ts/project";
	import type { Project } from '$lib/ts/drive';

    import Navbar from '$lib/components/Navbar.svelte';

    import '$lib/assets/style/drive.css';
    
    import Toast from '$lib/components/Toast.svelte';
    
    import ShareModal from '$lib/components/ShareModal.svelte';

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
    
    // Add these variables for breadcrumb navigation
    let currentView = 'drive';
    let displayedDocuments: Document[] = [];
    
    // Add this near your other variable declarations
    let projectDocumentsMap = new Map<string, number[]>();
    
    // Add these variables for toast notifications
    type ToastData = {
        message: string;
        type: 'success' | 'error' | 'warning';
    };
    
    let toasts: ToastData[] = [];
    
    // Add these new variables
    let shareModalOpen = false;
    let shareModalType: 'document' | 'project' = 'document';
    let shareModalId: number = 0;
    let shareModalTitle = '';

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
            
            // Initialize displayed documents
            updateDisplayedDocuments();
            
            console.log("Documents loaded:", documents);
            console.log("Projects loaded:", projects);
		} catch (error) {
            console.error("Error loading drive content:", error);
		} finally {
			isLoading = false;
		}
	});

    // Add this function to update displayed documents based on current view and filters
    function updateDisplayedDocuments() {
        if (currentProject && currentProject.id) {
            // Show only documents in the current project
            displayedDocuments = documents.filter(doc => 
                projectDocumentsMap.get(currentProject?.id ?? '')?.includes(doc.id) ?? false
            );
        } else if (activeCategory === 'starred') {
            // Show only starred documents that are not trashed
            displayedDocuments = starredDocuments.filter(doc => !doc.is_trashed);
        } else if (activeCategory === 'trash') {
            // Show only trashed documents
            displayedDocuments = trashedDocuments;
        } else {
            // Default view - show only documents that are not in any project and not trashed
            displayedDocuments = documents.filter(doc => {
                // Skip trashed documents
                if (doc.is_trashed) return false;
                
                // Check if document is in any project
                for (const [_, projectDocs] of projectDocumentsMap) {
                    if (projectDocs.includes(doc.id)) return false;
                }
                
                return true;
            });
        }
    }
    
    async function setActiveCategory(category: string) {
		activeCategory = category;
        currentProject = null;
        currentView = 'drive';
        
        try {
            if (category === 'starred') {
                // Refresh starred items when switching to starred view
                const [starredDocsResult, starredProjsResult] = await Promise.all([
                    get_starred_documents(),
                    get_starred_projects()
                ]);
                
                starredDocuments = starredDocsResult || [];
                starredProjects = starredProjsResult || [];
            } else if (category === 'all') {
                // Refresh all items when switching to main view
                const [docsResult, projectsResult, starredDocsResult, starredProjsResult] = await Promise.all([
                    get_all_documents(),
                    get_all_projects(),
                    get_starred_documents(),
                    get_starred_projects()
                ]);
                
                documents = docsResult || [];
                projects = projectsResult || [];
                starredDocuments = starredDocsResult || [];
                starredProjects = starredProjsResult || [];
            }
        } catch (error) {
            console.error("Error refreshing items:", error);
        }
        
        // Update displayed documents based on the selected category
        updateDisplayedDocuments();
    }
    
    // Function to show a toast notification
    function showToast(message: string, type: 'success' | 'error' | 'warning' = 'success') {
        toasts = [...toasts, { message, type }];
    }
    
    // Function to remove a toast
    function removeToast(index: number) {
        toasts = toasts.filter((_, i) => i !== index);
    }
    
    // Handle new project creation
    async function handleCreateProject() {
        if (!newProjectName.trim()) {
            showToast('Project name cannot be empty', 'error');
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
                
                // Show success toast
                showToast(`Project "${project.name}" created successfully`);
            } else {
                showToast('Failed to create project', 'error');
            }
        } catch (error) {
            console.error('Error creating project:', error);
            showToast('An error occurred while creating the project', 'error');
        }
    }
    
    // Handle new document creation
    async function handleCreateDocument() {
        if (!newDocumentName.trim()) {
            showToast('Document name cannot be empty', 'error');
            return;
        }
        
        try {
            // Create a new document with the name and content
            const document = await create_document(newDocumentName, newDocumentContent || '');
            
            if (document) {
                // If we're in a project view or have a selected project ID, associate the document with the project
                const projectId = currentProject?.id || newDocumentProjectId;
                
                if (projectId) {
                    await add_document_to_project(parseInt(projectId), document.id);
                    
                    // If we're in a project view, update the project documents map
                    if (currentProject && currentProject.id) {
                        const currentDocs = projectDocumentsMap.get(currentProject.id) || [];
                        projectDocumentsMap.set(currentProject.id, [...currentDocs, document.id]);
                    }
                }
                
                // Refresh the documents list
                const refreshedDocs = await get_all_documents();
                documents = refreshedDocs || [];
                
                // Clear the form and close the modal
                newDocumentName = '';
                newDocumentContent = '';
                newDocumentProjectId = null;
                showNewDocumentModal = false;
                
                // Update displayed documents
                updateDisplayedDocuments();
                
                // Show success toast
                showToast(`Document "${document.name}" created successfully`);
            } else {
                showToast('Failed to create document', 'error');
            }
        } catch (error) {
            console.error('Error creating document:', error);
            showToast('An error occurred while creating the document', 'error');
        }
    }
    
    // Handle project click
    async function handleProjectClick(project: Project) {
        currentProject = project;
        currentView = 'project';
        
        // Load project documents if not already loaded
        if (!projectDocumentsMap.has(project.id)) {
            try {
                const projectDocs = await get_project_documents(parseInt(project.id));
                if (projectDocs) {
                    projectDocumentsMap.set(project.id, projectDocs.map(doc => doc.id));
                }
            } catch (error) {
                console.error("Error loading project documents:", error);
            }
        }
        
        // Update displayed documents
        updateDisplayedDocuments();
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
    
    // Toggle star on document
    async function handleToggleStarDocument(event: Event, document: Document) {
        event.stopPropagation(); // Prevent document click
        
        const success = await toggle_star_document(document);
        if (success) {
            // Update the document's starred status locally
            document.is_starred = !document.is_starred;
            
            // Update starred documents list
            if (document.is_starred) {
                // Add to starred documents
                starredDocuments = [...starredDocuments, document];
                showToast(`Document "${document.name}" added to starred`, 'success');
            } else {
                // Remove from starred documents
                starredDocuments = starredDocuments.filter(d => d.id !== document.id);
                showToast(`Document "${document.name}" removed from starred`, 'success');
                
                // If we're in the starred view, remove this document from displayed documents
                if (activeCategory === 'starred') {
                    displayedDocuments = displayedDocuments.filter(d => d.id !== document.id);
                }
            }
            
            // Force UI update by creating a new array reference
            documents = [...documents];
            
            // Also update the document in displayedDocuments to ensure the UI updates
            if (activeCategory !== 'starred' || document.is_starred) {
                displayedDocuments = displayedDocuments.map(d => 
                    d.id === document.id ? {...document} : d
                );
            }
        } else {
            showToast('Failed to update star status', 'error');
        }
    }
    
    // Toggle star on project
    async function handleToggleStarProject(event: Event, project: Project) {
        event.stopPropagation(); // Prevent project click
        
        const success = await toggle_star_project(parseInt(project.id));
        if (success) {
            // Update the project's starred status locally
            project.is_starred = !project.is_starred;
            
            // Update starred projects list
            if (project.is_starred) {
                starredProjects = [...starredProjects, project];
                showToast(`Project "${project.name}" added to starred`, 'success');
            } else {
                starredProjects = starredProjects.filter(p => p.id !== project.id);
                showToast(`Project "${project.name}" removed from starred`, 'success');
            }
            
            // Force UI update
            projects = [...projects];
        } else {
            showToast('Failed to update star status', 'error');
        }
    }
    
    // Move document to trash
    async function handleTrashDocument(event: Event, document: Document) {
        event.stopPropagation(); // Prevent document click
        
        const success = await trash_document(document);
        if (success) {
            // Update the document's trashed status locally
            document.is_trashed = true;
            
            // Add to trashed documents list
            trashedDocuments = [...trashedDocuments, document];
            
            // Remove from main documents list
            documents = documents.filter(d => d.id !== document.id);
            
            // Remove from starred documents list if it was starred
            starredDocuments = starredDocuments.filter(d => d.id !== document.id);
            
            // Update displayed documents
            if (activeCategory !== 'trash') {
                displayedDocuments = displayedDocuments.filter(d => d.id !== document.id);
            } else {
                // If we're in trash view, add it to displayed documents
                displayedDocuments = [...displayedDocuments, document];
            }
            
            showToast(`Document "${document.name}" moved to trash`, 'success');
        } else {
            showToast('Failed to move document to trash', 'error');
        }
    }
    
    // Move project to trash
    async function handleTrashProject(event: Event, project: Project) {
        event.stopPropagation(); // Prevent project click
        
        const success = await trash_project(parseInt(project.id));
        if (success) {
            // Update the project's trashed status locally
            project.is_trashed = true;
            
            // Add to trashed projects list
            trashedProjects = [...trashedProjects, project];
            
            // Remove from displayed projects
            projects = projects.filter(p => p.id !== project.id);
            
            // Remove from starred projects list if it was starred
            starredProjects = starredProjects.filter(p => p.id !== project.id);
            
            showToast(`Project "${project.name}" moved to trash`, 'success');
        } else {
            showToast('Failed to move project to trash', 'error');
        }
    }
    
    // Restore document from trash
    async function handleRestoreDocument(event: Event, document: Document) {
        event.stopPropagation(); // Prevent document click
        
        const success = await restore_document(document);
        if (success) {
            // Update the document's trashed status locally
            document.is_trashed = false;
            
            // Add back to main documents list
            documents = [...documents, document];
            
            // Remove from trashed documents list
            trashedDocuments = trashedDocuments.filter(d => d.id !== document.id);
            
            // Update displayed documents to refresh the view
            updateDisplayedDocuments();
            
            showToast(`Document "${document.name}" restored from trash`, 'success');
        } else {
            showToast('Failed to restore document', 'error');
        }
    }
    
    // Restore project from trash
    async function handleRestoreProject(event: Event, project: Project) {
        event.stopPropagation(); // Prevent project click
        
        const success = await restore_project(parseInt(project.id));
        if (success) {
            // Update the project's trashed status locally
            project.is_trashed = false;
            
            // Add back to main projects list
            projects = [...projects, project];
            
            // Remove from trashed projects list
            trashedProjects = trashedProjects.filter(p => p.id !== project.id);
            
            // Update displayed documents to refresh the view
            updateDisplayedDocuments();
            
            showToast(`Project "${project.name}" restored from trash`, 'success');
        } else {
            showToast('Failed to restore project', 'error');
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
            // Check if document is already in the project
            const currentDocs = projectDocumentsMap.get(project.id) || [];
            if (currentDocs.includes(draggedDocument.id)) {
                // Show warning toast
                showToast(`Document "${draggedDocument.name}" is already in project "${project.name}"`, 'warning');
                draggedDocument = null;
                return;
            }
            
            // Add document to project
            const success = await add_document_to_project(parseInt(project.id), draggedDocument.id);
            
            if (success) {
                // Show success toast
                showToast(`Document "${draggedDocument.name}" added to project "${project.name}"`, 'success');
                
                // Update the project documents map
                projectDocumentsMap.set(project.id, [...currentDocs, draggedDocument.id]);
                
                // If we're currently viewing this project, update the displayed documents
                if (currentProject && currentProject.id === project.id) {
                    updateDisplayedDocuments();
                }
            } else {
                // Show error toast
                showToast(`Failed to add document to project`, 'error');
            }
            
            draggedDocument = null;
        }
    }
    
    // Add function to return to main drive view
    function returnToDrive() {
        currentProject = null;
        currentView = 'drive';
        updateDisplayedDocuments();
    }
    
    // Update category change handler to reset project view
    function changeCategory(category: string) {
        activeCategory = category;
        currentProject = null;
        currentView = 'drive';
        updateDisplayedDocuments();
    }

    // Add this new function
    function openShareModal(type: 'document' | 'project', id: number, title: string) {
        shareModalType = type;
        shareModalId = id;
        shareModalTitle = title;
        shareModalOpen = true;
    }

    function closeShareModal() {
        shareModalOpen = false;
	}
</script>

{#each toasts as toast, i}
    <Toast 
        message={toast.message} 
        type={toast.type} 
        onClose={() => removeToast(i)} 
    />
{/each}

<div class="bg-black min-vh-100 d-flex flex-column">
	<Navbar {isLoggedIn} />

	<div class="container-fluid flex-grow-1 d-flex">
		<div class="row flex-grow-1 w-100 m-0">
			<!-- Left Sidebar -->
			<div class="col-md-2 p-0 sidebar-column">
				<div class="sidebar bg-dark border-end border-dark">
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

					<div class="sidebar-bottom">
						<!-- Create buttons at bottom -->
						<div class="p-3 border-top border-dark">
							<button class="btn btn-green w-100 mb-2" on:click={() => showNewProjectModal = true}>
								<i class="bi bi-folder-plus me-2"></i> New Project
							</button>
							<button class="btn btn-outline-light w-100" on:click={() => showNewDocumentModal = true}>
								<i class="bi bi-file-earmark-plus me-2"></i> New Document
							</button>
						</div>
					</div>
				</div>
			</div>

			<!-- Main Content Area -->
			<div class="col-md-10 bg-black p-4 content-column">
				<h1 class="mb-4">My Drive</h1>
                
                <!-- Add breadcrumb navigation with custom styling -->
                <nav aria-label="breadcrumb" class="mb-4">
                    <ol class="breadcrumb">
                        <li class="breadcrumb-item">
                            <button 
                                class="btn btn-link p-0 {!currentProject ? 'text-green' : 'text-white'}" 
                                on:click={returnToDrive}>
                                Drive
                            </button>
                        </li>
                        {#if currentProject}
                            <li class="breadcrumb-item active text-green" aria-current="page">
                                {currentProject.name}
                            </li>
                        {/if}
                    </ol>
                </nav>

				{#if isLoading}
					<div class="d-flex justify-content-center my-5">
						<div class="spinner-border text-green" role="status">
							<span class="visually-hidden">Loading...</span>
						</div>
					</div>
				{:else}
					<!-- Unified Items Section -->
                    <div class="mt-11 mb-4">
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
                                {#if !currentProject && activeCategory === 'trash'}
                                    {#each trashedProjects as project}
                                        <div class="col">
                                            <div 
                                                class="card bg-dark border-0 h-100 project-card" 
                                                role="button"
                                                tabindex="0"
                                                on:click={() => handleProjectClick(project)}
                                                on:keydown={(e) => e.key === 'Enter' && handleProjectClick(project)}>
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
                                                
                                                <!-- Action icons for trashed projects -->
                                                <div class="card-actions">
                                                    <button class="action-icon restore-icon" on:click={(e) => handleRestoreProject(e, project)} title="Restore" aria-label="Restore project from trash">
                                                        <i class="bi bi-arrow-counterclockwise"></i>
                                                    </button>
                                                </div>
                                            </div>
                                        </div>
                                    {/each}
                                {:else if !currentProject && activeCategory === 'starred'}
                                    {#each starredProjects as project}
                                        <div class="col">
                                            <div 
                                                class="card bg-dark border-0 h-100 project-card" 
                                                role="button"
                                                tabindex="0"
                                                on:click={() => handleProjectClick(project)}
                                                on:keydown={(e) => e.key === 'Enter' && handleProjectClick(project)}
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
                                                
                                                <!-- Action icons for starred projects -->
                                                <div class="card-actions">
                                                    <button 
                                                        class="action-icon star-icon" 
                                                        on:click={(e) => handleToggleStarProject(e, project)} 
                                                        aria-label="Unstar project"
                                                    >
                                                        <i class="bi bi-star-fill text-warning"></i>
                                                    </button>
                                                    <button 
                                                        class="action-icon share-icon" 
                                                        on:click={(e) => { e.stopPropagation(); openShareModal('project', parseInt(project.id), project.name); }}
                                                        aria-label="Share project"
                                                    >
                                                        <i class="bi bi-share"></i>
                                                    </button>
                                                    <button 
                                                        class="action-icon trash-icon" 
                                                        on:click={(e) => handleTrashProject(e, project)} 
                                                        aria-label="Move project to trash"
                                                    >
                                                        <i class="bi bi-trash"></i>
                                                    </button>
                                                </div>
                                            </div>
                                        </div>
                                    {/each}
                                {:else if !currentProject && activeCategory !== 'trash' && activeCategory !== 'starred'}
								{#each projects as project}
									<div class="col">
                                            <div 
                                                class="card bg-dark border-0 h-100 project-card" 
                                                role="button"
                                                tabindex="0"
                                                on:click={() => handleProjectClick(project)}
                                                on:keydown={(e) => e.key === 'Enter' && handleProjectClick(project)}
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
                                                        <button class="action-icon restore-icon" on:click={(e) => handleRestoreProject(e, project)} title="Restore" aria-label="Restore project from trash">
                                                            <i class="bi bi-arrow-counterclockwise"></i>
                                                        </button>
                                                    {:else}
                                                        <button 
                                                            class="action-icon star-icon" 
                                                            on:click={(e) => handleToggleStarProject(e, project)} 
                                                            aria-label={project.is_starred ? "Unstar project" : "Star project"}
                                                        >
                                                            <i class="bi {project.is_starred ? 'bi-star-fill text-warning' : 'bi-star'}"></i>
                                                        </button>
                                                        <button 
                                                            class="action-icon share-icon" 
                                                            on:click={(e) => { e.stopPropagation(); openShareModal('project', parseInt(project.id), project.name); }}
                                                            aria-label="Share project"
                                                        >
                                                            <i class="bi bi-share"></i>
                                                        </button>
                                                        <button 
                                                            class="action-icon trash-icon" 
                                                            on:click={(e) => handleTrashProject(e, project)} 
                                                            aria-label="Move project to trash"
                                                        >
                                                            <i class="bi bi-trash"></i>
                                                        </button>
                                                    {/if}
											</div>
										</div>
									</div>
								{/each}
                                {/if}

								<!-- Documents After Projects -->
                                {#each displayedDocuments as document}
									<div class="col">
                                        <div 
                                            class="card bg-dark border-0 h-100 document-card" 
                                            role="button"
                                            tabindex="0"
                                            on:click={() => handleDocumentClick(document)}
                                            on:keydown={(e) => e.key === 'Enter' && handleDocumentClick(document)}
                                            draggable={!currentProject}
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
                                                    <button class="action-icon restore-icon" on:click={(e) => handleRestoreDocument(e, document)} title="Restore" aria-label="Restore document from trash">
                                                        <i class="bi bi-arrow-counterclockwise"></i>
                                                    </button>
                                                {:else}
                                                    <button 
                                                        class="action-icon star-icon" 
                                                        on:click={(e) => handleToggleStarDocument(e, document)} 
                                                        aria-label={document.is_starred ? "Unstar document" : "Star document"}
                                                    >
                                                        <i class="bi {document.is_starred ? 'bi-star-fill text-warning' : 'bi-star'}"></i>
                                                    </button>
                                                    <button 
                                                        class="action-icon share-icon" 
                                                        on:click={(e) => { e.stopPropagation(); openShareModal('document', document.id, document.name); }}
                                                        aria-label="Share document"
                                                    >
                                                        <i class="bi bi-share"></i>
                                                    </button>
                                                    <button 
                                                        class="action-icon trash-icon" 
                                                        on:click={(e) => handleTrashDocument(e, document)} 
                                                        aria-label="Move document to trash"
                                                    >
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

<!-- ShareModal component -->
<ShareModal
    bind:isOpen={shareModalOpen}
    type={shareModalType}
    id={shareModalId}
    title={shareModalTitle}
    on:close={closeShareModal}
    on:toast={({ detail }) => showToast(detail.message, detail.type)}
    on:confirm={({ detail }) => {
        if (confirm(detail.message)) {
            detail.onConfirm();
        } else {
            detail.onCancel && detail.onCancel();
        }
    }}
/>

<!-- Add these styles -->
<style>
    /* Navbar fixed positioning */
    :global(nav.navbar) {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        z-index: 1100;
    }
    
    /* Add padding to account for fixed navbar */
    .container-fluid {
        padding-top: 70px; /* Height of navbar */
    }

    /* Sidebar fixed positioning */
    .sidebar-column {
        position: relative;
        height: 100vh;
        overflow: hidden;
    }

    .sidebar {
        position: fixed;
        width: inherit;
        max-width: 16.666%; /* Same as col-md-2 */
        height: calc(100vh - 70px); /* Subtract navbar height */
        display: flex;
        flex-direction: column;
        overflow-y: auto;
        z-index: 1000;
        top: 70px; /* Position below navbar */
    }

    .sidebar-bottom {
        margin-top: auto;
    }

    /* Content column styling */
    .content-column {
        height: calc(100vh - 70px); /* Subtract navbar height */
        overflow-y: auto;
        margin-top: 70px; /* Add top margin to account for navbar */
    }

    /* Active nav styling */
    .nav-link.active {
        font-weight: 500;
    }

    /* Progress bar color */
    .bg-green {
        background-color: #198754;
    }

    /* Custom button styles */
    .btn-green {
        background-color: #198754;
        border-color: #198754;
        color: white;
    }

    .btn-green:hover {
        background-color: #157347;
        border-color: #146c43;
        color: white;
    }

    .card {
        position: relative;
        overflow: visible;
        transition: all 0.2s ease;
    }

    .card:hover {
        transform: translateY(-2px);
    }

    .card-actions {
        position: absolute;
        top: 0.5rem;
        right: 0.5rem;
        display: flex;
        gap: 0.5rem;
        opacity: 0;
        transition: opacity 0.2s ease;
        z-index: 10;
        background: rgba(0, 0, 0, 0.7);
        padding: 0.25rem;
        border-radius: 4px;
    }

    .card:hover .card-actions {
        opacity: 1;
    }

    .action-icon {
        background: none;
        border: none;
        padding: 0.25rem;
		cursor: pointer;
        color: var(--color-text-secondary);
        transition: color 0.2s ease;
        font-size: 1rem;
        display: flex;
        align-items: center;
        justify-content: center;
        width: 28px;
        height: 28px;
        border-radius: 4px;
    }

    .action-icon:hover {
        color: var(--color-primary);
        background: rgba(255, 255, 255, 0.1);
    }

    .trash-icon:hover {
        color: var(--color-error);
    }

    .share-icon:hover {
        color: var(--color-primary);
    }

    /* Ensure the card body doesn't overlap with the actions */
    .card-body {
        position: relative;
        z-index: 1;
	}
</style>
