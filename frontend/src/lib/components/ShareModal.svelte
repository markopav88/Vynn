<script lang="ts">
    import { fade } from 'svelte/transition';
    import { createEventDispatcher } from 'svelte';
    import { onMount } from 'svelte';
    import profileImage from '$lib/assets/profile-image.png';
    import { get_document_permissions, type DocumentUser, add_document_permissions, update_document_permissions, delete_document_permissions } from '$lib/ts/document';
    import { get_project_permissions, type ProjectUser, add_project_permissions, update_project_permission, remove_project_permissions } from '$lib/ts/project';
    import { get_current_user } from '$lib/ts/user';

    export let isOpen = false;
    export let title = '';
    export let type: 'document' | 'project' = 'document';
    export let id: number;

    const dispatch = createEventDispatcher();

    let searchQuery = '';
    let loading = true;
    let usersWithAccess: (DocumentUser | ProjectUser)[] = [];
    let error = '';
    let searchResults: { id: number; email: string; name: string }[] = [];
    let isSearching = false;
    let searchTimeout: ReturnType<typeof setTimeout> | null = null;
    let selectedUser: { id: number; email: string; name: string } | null = null;
    let selectedRole = 'editor';
    let editingUser: (DocumentUser | ProjectUser) | null = null;
    let currentUserId: number = 2; // Hard-coding the current user ID to ensure it's set before rendering

    // Update the reactive statement for isOpen to wait for the userId to be set first
    $: if (isOpen) {
        // Don't call loadUsers directly, it will be called after user ID is loaded
        fetchData();
    }

    async function searchUsers(query: string): Promise<void> {
        if (!query.trim()) {
            searchResults = [];
            return;
        }

        try {
            isSearching = true;
            const response = await fetch(`http://localhost:3001/api/users/search?q=${encodeURIComponent(query)}`, {
                credentials: 'include'
            });

            if (!response.ok) {
                throw new Error('Failed to search users');
            }

            const data = await response.json();
            // Filter out users that already have access
            searchResults = data.filter((user: { id: number }) => 
                !usersWithAccess.some(existingUser => existingUser.id === user.id)
            );
        } catch (err) {
            console.error('Error searching users:', err);
            searchResults = [];
        } finally {
            isSearching = false;
        }
    }

    function handleSearchInput(event: Event): void {
        const input = event.target as HTMLInputElement;
        searchQuery = input.value;

        // Clear previous timeout
        if (searchTimeout) {
            clearTimeout(searchTimeout);
        }

        // Set new timeout for debouncing
        searchTimeout = setTimeout(() => {
            searchUsers(searchQuery);
        }, 300);
    }

    async function handleUserSelect(user: { id: number; email: string; name: string }): Promise<void> {
        selectedUser = user;
    }

    async function handleAddUser(): Promise<void> {
        if (!selectedUser) return;

        try {
            let success = false;
            if (type === 'document') {
                success = await add_document_permissions(id, selectedUser.id, selectedRole);
            } else {
                success = await add_project_permissions(id, selectedUser.id, selectedRole);
            }

            if (success) {
                // Refresh the users list
                await loadUsers();
                // Clear search
                searchQuery = '';
                searchResults = [];
                selectedUser = null;
                selectedRole = 'editor';
            } else {
                error = 'Failed to add user';
            }
        } catch (err) {
            console.error('Error adding user:', err);
            error = 'Failed to add user';
        }
    }

    async function handleRoleChange(user: DocumentUser | ProjectUser, newRole: string): Promise<void> {
        try {
            // Get the user ID from either user_id or id property
            const userId = (user as any).user_id || user.id;
            if (!userId) {
                console.error('Invalid user object:', user);
                dispatch('toast', { message: 'Failed to update user permissions', type: 'error' });
                return;
            }

            // Prevent changing own permissions
            if (isCurrentUser(user)) {
                console.error('Cannot modify own permissions');
                dispatch('toast', { message: 'You cannot modify your own permissions', type: 'error' });
                return;
            }

            // Check if this is an ownership change
            if (newRole === 'owner') {
                const currentOwner = usersWithAccess.find(u => u.role === 'owner');
                if (currentOwner && currentOwner.id !== userId) {
                    // Show confirmation toast
                    dispatch('confirm', { 
                        message: `Are you sure you want to transfer ownership to ${user.name}? You will become an editor.`,
                        onConfirm: async () => {
                            try {
                                if (type === 'document') {
                                    await update_document_permissions(id, userId, newRole);
                                } else {
                                    await update_project_permission(id, userId, newRole);
                                }
                                await loadUsers();
                                editingUser = null;
                                dispatch('toast', { message: `Ownership transferred to ${user.name}`, type: 'success' });
                            } catch (err: any) {
                                console.error('Error updating role:', err);
                                dispatch('toast', { 
                                    message: 'Failed to update user permissions', 
                                    type: 'error' 
                                });
                            }
                        },
                        onCancel: () => {
                            dispatch('toast', { message: 'Ownership transfer cancelled', type: 'warning' });
                        }
                    });
                    return;
                }
            }

            const resetUserRole = () => {
                const currentRole = usersWithAccess.find(u => {
                    return (u as any).user_id === userId || u.id === userId;
                })?.role || user.role;
                user.role = currentRole;
            };

            try {
                if (type === 'document') {
                    await update_document_permissions(id, userId, newRole);
                } else {
                    await update_project_permission(id, userId, newRole);
                }
                await loadUsers();
                editingUser = null;
                dispatch('toast', { message: 'Permissions updated successfully', type: 'success' });
            } catch (err: any) {
                console.error('Error updating role:', err);
                dispatch('toast', { 
                    message: 'Failed to update user permissions', 
                    type: 'error' 
                });
                // Reset the role to its previous value
                resetUserRole();
            }
        } catch (err: any) {
            console.error('Error in handleRoleChange:', err);
            dispatch('toast', { 
                message: 'Failed to update user permissions', 
                type: 'error' 
            });
            // We can't use userId here because it's defined in the scope of the try block
            const resetId = (user as any).user_id || user.id;
            const currentRole = usersWithAccess.find(u => {
                return (u as any).user_id === resetId || u.id === resetId;
            })?.role || user.role;
            user.role = currentRole;
        }
    }

    async function handleRemoveUser(user: DocumentUser | ProjectUser): Promise<void> {
        try {
            // Get the user ID from either user_id or id property
            const userId = (user as any).user_id || user.id;
            if (!userId) {
                console.error('Invalid user object:', user);
                dispatch('toast', { message: 'Failed to update user permissions', type: 'error' });
                return;
            }
            
            // Prevent removing self
            if (isCurrentUser(user)) {
                console.error('Cannot remove self from permissions');
                dispatch('toast', { message: 'You cannot remove yourself', type: 'error' });
                return;
            }

            try {
                if (type === 'document') {
                    await delete_document_permissions(id, userId);
                } else {
                    await remove_project_permissions(id, userId);
                }
                await loadUsers();
                dispatch('toast', { message: 'User removed successfully', type: 'success' });
            } catch (err: any) {
                console.error('Error removing user:', err);
                dispatch('toast', { 
                    message: 'Failed to update user permissions', 
                    type: 'error' 
                });
            }
        } catch (err: any) {
            console.error('Error in handleRemoveUser:', err);
            dispatch('toast', { 
                message: 'Failed to update user permissions', 
                type: 'error' 
            });
        }
    }

    async function loadUsers(): Promise<void> {
        try {
            loading = true;
            error = '';
            
            if (type === 'document') {
                const permissions = await get_document_permissions(id);
                if (permissions) {
                    usersWithAccess = permissions;
                }
            } else {
                const permissions = await get_project_permissions(id);
                if (permissions) {
                    usersWithAccess = permissions;
                }
            }
        } catch (err) {
            console.error('Error loading permissions:', err);
            error = 'Failed to load users with access';
        } finally {
            loading = false;
        }
    }

    function closeModal(): void {
        isOpen = false;
        dispatch('close');
    }

    function handleModalClick(event: MouseEvent): void {
        // Only close if clicking directly on the backdrop button itself
        closeModal();
    }

    // New async function to coordinate loading
    async function fetchData() {
        try {
            const userData = await get_current_user();
            if (userData && userData.id) {
                currentUserId = Number(userData.id);
            }
        } catch (err) {
            console.error('Error getting current user:', err);
        }
        
        // Load users with permissions
        await loadUsers();
    }

    // Simplify isCurrentUser function
    function isCurrentUser(user: DocumentUser | ProjectUser): boolean {
        if (typeof currentUserId !== 'number') {
            return false;
        }
        
        // Get user ID considering both possible properties
        const userId = Number((user as any).user_id || user.id);
        
        // Simple comparison
        return userId === currentUserId;
    }

    // Add this back
    onMount(() => {
        if (isOpen) {
            // Initialize on mount
            fetchData();
        }
    });
</script>

{#if isOpen}
    <section 
        class="modal-wrapper"
        role="dialog"
        aria-modal="true"
        aria-labelledby="modal-title"
    >
        <button 
            class="modal-overlay" 
            on:click={closeModal}
            aria-label="Close modal"
        ></button>
        <div 
            class="modal-content" 
            transition:fade
            role="document"
        >
            <div class="modal-header">
                <h2 id="modal-title">Share {type === 'document' ? 'Document' : 'Project'}: {title}</h2>
                <button 
                    class="close-button" 
                    on:click={closeModal} 
                    aria-label="Close modal"
                    on:keydown={(e) => e.key === 'Escape' && closeModal()}
                >×</button>
            </div>

            <div class="modal-body">
                <div class="search-container">
                    <input
                        type="text"
                        bind:value={searchQuery}
                        on:input={handleSearchInput}
                        placeholder="Search users by email..."
                        class="search-input"
                    />
                    {#if isSearching}
                        <div class="search-loading">Searching...</div>
                    {/if}
                    {#if searchResults.length > 0}
                        <div class="search-results" role="listbox">
                            {#each searchResults as user}
                                <button 
                                    class="search-result-item {selectedUser?.id === user.id ? 'selected' : ''}" 
                                    on:click={() => handleUserSelect(user)}
                                    role="option"
                                    aria-selected={selectedUser?.id === user.id}
                                    type="button"
                                >
                                    <span class="user-email">{user.email}</span>
                                    <span class="user-name">{user.name}</span>
                                </button>
                            {/each}
                        </div>
                    {/if}
                </div>

                {#if selectedUser}
                    <div class="selected-user-container">
                        <div class="selected-user">
                            <span class="user-email">{selectedUser.email}</span>
                            <span class="user-name">{selectedUser.name}</span>
                        </div>
                        <div class="role-selector">
                            <select bind:value={selectedRole}>
                                <option value="viewer">Viewer</option>
                                <option value="editor">Editor</option>
                                {#if type === 'project' || type === 'document'}
                                    <option value="owner">Owner</option>
                                {/if}
                            </select>
                            <button class="add-button" on:click={handleAddUser}>Add User</button>
                        </div>
                    </div>
                {/if}

                {#if error}
                    <div class="error-message">{error}</div>
                {/if}

                {#if loading}
                    <div class="loading">Loading users...</div>
                {:else}
                    <div class="users-list">
                        {#each usersWithAccess as user}
                            <div class="user-item">
                                <img src={profileImage} alt="Profile" class="profile-image" />
                                <div class="user-info">
                                    <span class="user-name">{user.name}</span>
                                    <span class="user-email">{user.email}</span>
                                </div>
                                <div class="user-role">
                                    {#if isCurrentUser(user)}
                                        <span class="role-badge {user.role.toLowerCase()} current-user">
                                            {user.role}
                                        </span>
                                    {:else if editingUser?.id === user.id}
                                        <select 
                                            bind:value={user.role}
                                            on:change={() => handleRoleChange(user, user.role)}
                                            class="role-dropdown"
                                        >
                                            <option value="viewer">Viewer</option>
                                            <option value="editor">Editor</option>
                                            {#if type === 'project' || type === 'document'}
                                                <option value="owner">Owner</option>
                                            {/if}
                                        </select>
                                    {:else}
                                        <span 
                                            class="role-badge {user.role.toLowerCase()}"
                                            on:click={() => editingUser = user}
                                            on:keydown={(e) => e.key === 'Enter' && (editingUser = user)}
                                            tabindex={0}
                                            role="button"
                                            aria-label="Edit role"
                                        >
                                            {user.role}
                                        </span>
                                    {/if}
                                </div>
                                {#if !isCurrentUser(user)}
                                    <button 
                                        class="remove-button"
                                        on:click={() => handleRemoveUser(user)}
                                        aria-label="Remove user"
                                    >
                                        ×
                                    </button>
                                {:else}
                                    <div class="remove-button-placeholder"></div>
                                {/if}
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>
        </div>
    </section>
{/if}

<style>
    .modal-wrapper {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 1000;
    }

    .modal-overlay {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background-color: rgba(0, 0, 0, 0.5);
        cursor: pointer;
        border: none;
        padding: 0;
        margin: 0;
        width: 100%;
        height: 100%;
    }

    .modal-content {
        background-color: #1a1a1a;
        border-radius: 8px;
        width: 500px;
        max-width: 90%;
        max-height: 90vh;
        overflow-y: auto;
        color: white;
        position: relative;
        z-index: 1001;
    }

    .modal-header {
        padding: 1rem;
        border-bottom: 1px solid #333;
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .close-button {
        background: none;
        border: none;
        color: white;
        font-size: 1.5rem;
        cursor: pointer;
        padding: 0.5rem;
    }

    .modal-body {
        padding: 1rem;
    }

    .search-container {
        position: relative;
        margin-bottom: 1rem;
    }

    .search-input {
        width: 100%;
        padding: 0.5rem;
        border: 1px solid #333;
        border-radius: 4px;
        background-color: #2a2a2a;
        color: white;
        font-size: 1rem;
    }

    .search-results {
        position: absolute;
        top: 100%;
        left: 0;
        right: 0;
        background-color: #2a2a2a;
        border: 1px solid #333;
        border-radius: 4px;
        max-height: 200px;
        overflow-y: auto;
        z-index: 1001;
    }

    .search-result-item {
        padding: 0.5rem;
        cursor: pointer;
        display: flex;
        flex-direction: column;
        width: 100%;
        text-align: left;
        background: none;
        border: none;
        color: inherit;
    }

    .search-result-item:hover {
        background-color: #3a3a3a;
    }

    .search-result-item.selected {
        background-color: #3a3a3a;
    }

    .user-email {
        color: #888;
        font-size: 0.9rem;
    }

    .user-name {
        color: white;
        font-weight: 500;
    }

    .search-loading {
        position: absolute;
        right: 0.5rem;
        top: 50%;
        transform: translateY(-50%);
        color: #888;
    }

    .selected-user-container {
        background-color: #2a2a2a;
        padding: 1rem;
        border-radius: 4px;
        margin-bottom: 1rem;
    }

    .selected-user {
        margin-bottom: 1rem;
    }

    .role-selector {
        display: flex;
        gap: 1rem;
        align-items: center;
    }

    .role-selector select {
        padding: 0.5rem 2rem 0.5rem 1rem;
        border: 1px solid #333;
        border-radius: 4px;
        background-color: #1a1a1a;
        color: white;
        font-size: 0.9rem;
        appearance: none;
        background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' fill='white' viewBox='0 0 16 16'%3E%3Cpath d='M7.247 11.14L2.451 5.658C1.885 5.013 2.345 4 3.204 4h9.592a1 1 0 0 1 .753 1.659l-4.796 5.48a1 1 0 0 1-1.506 0z'/%3E%3C/svg%3E");
        background-repeat: no-repeat;
        background-position: right 0.5rem center;
        background-size: 12px;
        cursor: pointer;
        transition: all 0.2s ease;
    }

    .role-selector select:hover {
        border-color: #444;
        background-color: #2a2a2a;
    }

    .role-selector select:focus {
        outline: none;
        border-color: #4CAF50;
        box-shadow: 0 0 0 2px rgba(76, 175, 80, 0.2);
    }

    .role-selector select option {
        background-color: #1a1a1a;
        color: white;
        padding: 0.5rem;
    }

    .role-selector select option:hover {
        background-color: #2a2a2a;
    }

    .role-selector select option:selected {
        background-color: #4CAF50;
        color: white;
    }

    .add-button {
        padding: 0.5rem 1rem;
        background-color: #4CAF50;
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
    }

    .add-button:hover {
        background-color: #45a049;
    }

    .users-list {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .user-item {
        display: flex;
        align-items: center;
        padding: 0.5rem;
        background-color: #2a2a2a;
        border-radius: 4px;
    }

    .profile-image {
        width: 40px;
        height: 40px;
        border-radius: 50%;
        margin-right: 1rem;
    }

    .user-info {
        flex: 1;
        display: flex;
        flex-direction: column;
    }

    .user-role {
        margin-left: auto;
        margin-right: 1rem;
    }

    .role-dropdown {
        padding: 0.25rem 1.5rem 0.25rem 0.5rem;
        border: 1px solid #333;
        border-radius: 4px;
        background-color: #1a1a1a;
        color: white;
        font-size: 0.8rem;
        appearance: none;
        background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='10' fill='white' viewBox='0 0 16 16'%3E%3Cpath d='M7.247 11.14L2.451 5.658C1.885 5.013 2.345 4 3.204 4h9.592a1 1 0 0 1 .753 1.659l-4.796 5.48a1 1 0 0 1-1.506 0z'/%3E%3C/svg%3E");
        background-repeat: no-repeat;
        background-position: right 0.25rem center;
        background-size: 10px;
        cursor: pointer;
        transition: all 0.2s ease;
        min-width: 100px;
    }

    .role-dropdown:hover {
        border-color: #444;
        background-color: #2a2a2a;
    }

    .role-dropdown:focus {
        outline: none;
        border-color: #4CAF50;
        box-shadow: 0 0 0 2px rgba(76, 175, 80, 0.2);
    }

    .role-dropdown option {
        background-color: #1a1a1a;
        color: white;
        padding: 0.5rem;
    }

    .role-dropdown option:hover {
        background-color: #2a2a2a;
    }

    .role-dropdown option:selected {
        background-color: #4CAF50;
        color: white;
    }

    .role-badge {
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        font-size: 0.8rem;
        text-transform: capitalize;
        cursor: pointer;
        transition: all 0.2s ease;
        display: inline-block;
        min-width: 60px;
        text-align: center;
    }

    .role-badge.owner {
        background-color: #4a4a4a;
        color: #fff;
    }

    .role-badge.editor {
        background-color: #2a4a2a;
        color: #8f8;
    }

    .role-badge.viewer {
        background-color: #2a2a4a;
        color: #88f;
    }

    .role-badge.current-user {
        cursor: not-allowed;
        opacity: 0.8;
        position: relative;
    }

    .role-badge.current-user::after {
        content: '(You)';
        font-size: 0.7rem;
        margin-left: 0.5rem;
        opacity: 0.7;
    }

    .role-badge:hover:not(.current-user) {
        transform: translateY(-1px);
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
    }

    .role-badge.owner:hover:not(.current-user) {
        background-color: #5a5a5a;
    }

    .role-badge.editor:hover:not(.current-user) {
        background-color: #3a5a3a;
    }

    .role-badge.viewer:hover:not(.current-user) {
        background-color: #3a3a5a;
    }

    .remove-button {
        background: none;
        border: none;
        color: #ff4444;
        font-size: 1.2rem;
        cursor: pointer;
        padding: 0.25rem;
        margin-left: 0.5rem;
    }

    .remove-button:hover {
        color: #ff6666;
    }

    .remove-button-placeholder {
        width: 1.2rem;
        margin-left: 0.5rem;
    }

    .error-message {
        color: #ff4444;
        margin-bottom: 1rem;
    }

    .loading {
        text-align: center;
        color: #888;
    }
</style> 