<script lang="ts">
    import { onMount } from 'svelte';
    import { get_all_documents } from "$lib/ts/document";
    import { get_all_projects } from "$lib/ts/drive";
    import Navbar from '$lib/components/Navbar.svelte';
    import type { Document } from '$lib/ts/document';
    import type { Project } from '$lib/ts/drive';
    
    let isLoggedIn = true;
    let documents: Document[] = [];
    let projects: Project[] = [];
    let isLoading = true;
    let activeCategory = 'all';
    
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
    
    function createNewItem() {
        // This would open a modal or dropdown for creating new items
        console.log("Create new item clicked");
    }
</script>

<div class="bg-black min-vh-100 d-flex flex-column">
    <Navbar {isLoggedIn} />
    
    <div class="container-fluid flex-grow-1 d-flex">
        <div class="row flex-grow-1 w-100 m-0">
            <!-- Left Sidebar -->
            <div class="col-md-2 bg-dark p-0 border-end border-dark min-vh-100">
                <div class="d-flex flex-column h-100 sticky-top">
                    <!-- New Item Button -->
                    <div class="p-3 border-bottom border-dark">
                        <button class="btn btn-green rounded-circle" on:click={createNewItem} aria-label="Create new item">
                            <i class="bi bi-plus-lg"></i>
                        </button>
                    </div>
                    
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
                        </ul>
                    </div>
                    
                    <!-- Storage Info (at bottom) -->
                    <div class="mt-auto p-3 border-top border-dark">
                        <div class="progress mb-2" style="height: 8px;">
                            <div class="progress-bar bg-green" role="progressbar" style="width: 25%;" aria-valuenow="25" aria-valuemin="0" aria-valuemax="100"></div>
                        </div>
                        <small class="text-white-50">2.5 GB of 10 GB used</small>
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
                                <button class="btn btn-sm btn-outline-green me-2">
                                    <i class="bi bi-folder-plus me-1"></i> New Project
                                </button>
                                <button class="btn btn-sm btn-outline-light">
                                    <i class="bi bi-file-earmark-plus me-1"></i> New Document
                                </button>
                            </div>
                        </div>
                        
                        {#if projects.length === 0 && documents.length === 0}
                            <div class="text-white-50 p-5 text-center border border-dark rounded">
                                <i class="bi bi-inbox display-4 d-block mb-3"></i>
                                <p>No items found</p>
                                <div class="mt-3">
                                    <button class="btn btn-sm btn-outline-green me-2">Create Project</button>
                                    <button class="btn btn-sm btn-outline-light">Create Document</button>
                                </div>
                            </div>
                        {:else}
                            <div class="row row-cols-1 row-cols-md-4 g-4">
                                <!-- Projects First -->
                                {#each projects as project}
                                    <div class="col">
                                        <div class="card bg-dark border-0 h-100 item-card project-card">
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
                                        <div class="card bg-dark border-0 h-100 item-card document-card">
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