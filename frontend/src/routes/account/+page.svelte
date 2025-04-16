<script lang="ts">
    import { onMount } from 'svelte';
    import { get_current_user, update_user, upload_profile_image, get_profile_image_url } from '$lib/ts/user';
    import { get_all_commands, get_all_keybindings, add_update_keybinding, delete_keybinding, Command, UserKeybinding } from '$lib/ts/document';
    import Navbar from '$lib/components/Navbar.svelte';
    import profileDefault from '$lib/assets/profile-image.png';
    import '$lib/assets/style/account.css'
    
    let isLoggedIn = true;
    let isLoading = true;
    let isSaving = false;
    let successMessage = '';
    let errorMessage = '';
    
    // User data
    let userId: number;
    let name = '';
    let email = '';
    let password = '';
    let confirmPassword = '';
    
    // Profile image
    let profileImage = profileDefault;
    let imageFile: File | null = null;
    let imagePreview: string | null = null;
    
    // Active tab
    let activeTab = 'profile'; // 'profile' or 'keybindings'
    
    // Keybindings data
    let commands: Command[] = [];
    let userKeybindings: UserKeybinding[] = [];
    let isLoadingKeybindings = false;
    let keybindingsSuccessMessage = '';
    let keybindingsErrorMessage = '';
    let editingKeybinding: number | null = null;
    let newKeybindingValue = '';
    
    onMount(async () => {
        try {
            const user = await get_current_user();
            if (user) {
                userId = user.id;
                name = user.name;
                email = user.email;
                
                // Set profile image if available
                try {
                    // Create a unique URL with timestamp to prevent caching
                    const timestamp = new Date().getTime();
                    const imageUrl = `${get_profile_image_url(userId)}?t=${timestamp}`;
                    
                    // Check if the image exists
                    const response = await fetch(imageUrl, { method: 'HEAD' });
                    if (response.ok) {
                        profileImage = imageUrl;
                    }
                } catch (error) {
                    console.error('Error checking profile image:', error);
                }
                
                // Load data
                await loadKeybindings();
            } else {
                // Redirect to login if not logged in
                window.location.href = '/login';
            }
        } catch (error) {
            console.error('Error loading user data:', error);
            errorMessage = 'Failed to load user data';
        } finally {
            isLoading = false;
        }
    });
    
    // Handle profile image upload
    function handleFileSelect(event: Event) {
        const input = event.target as HTMLInputElement;
        if (!input.files || input.files.length === 0) {
            return;
        }
        
        const file = input.files[0];
        
        // Check if the file is an image
        if (!file.type.startsWith('image/')) {
            errorMessage = 'Please select an image file';
            return;
        }
        
        // Check file size (max 5MB)
        if (file.size > 5 * 1024 * 1024) {
            errorMessage = 'Image size must be less than 5MB';
            return;
        }
        
        imageFile = file;
        
        // Create a preview
        const reader = new FileReader();
        reader.onload = (e) => {
            imagePreview = e.target?.result as string;
        };
        reader.readAsDataURL(file);
        
        // Clear error message
        errorMessage = '';
    }
    
    // Handle form submission
    async function handleSubmit() {
        try {
            isSaving = true;
            errorMessage = '';
            successMessage = '';
            
            // Validate passwords match if changed
            if (password && password !== confirmPassword) {
                errorMessage = 'Passwords do not match';
                return;
            }
            
            // Update user information
            if (name && email) {
                const updated = await update_user(name, email, password);
                if (!updated) {
                    errorMessage = 'Failed to update user information';
                    return;
                }
            }
            
            // Upload profile image if selected
            if (imageFile) {
                const uploaded = await upload_profile_image(imageFile);
                if (!uploaded) {
                    errorMessage = 'Failed to upload profile image';
                    return;
                }
                
                // Update the profile image display
                profileImage = imagePreview || profileImage;
            }
            
            // Clear password fields
            password = '';
            confirmPassword = '';
            
            // Show success message
            successMessage = 'Account updated successfully';
            
            // Clear file input
            const fileInput = document.getElementById('profileImageInput') as HTMLInputElement;
            if (fileInput) {
                fileInput.value = '';
            }
            imageFile = null;
            imagePreview = null;
            
        } catch (error) {
            console.error('Error updating account:', error);
            errorMessage = 'An unexpected error occurred';
        } finally {
            isSaving = false;
        }
    }
    
    // Load keybindings data
    async function loadKeybindings() {
        try {
            isLoadingKeybindings = true;
            keybindingsErrorMessage = '';
            
            // Load commands and user keybindings in parallel
            const [cmdResult, keyResult] = await Promise.all([
                get_all_commands(),
                get_all_keybindings()
            ]);
            
            if (cmdResult) {
                commands = cmdResult;
            } else {
                keybindingsErrorMessage = 'Failed to load commands';
            }
            
            if (keyResult) {
                userKeybindings = keyResult;
            } else {
                // If user doesn't have custom keybindings yet, that's fine
                userKeybindings = [];
            }
            
        } catch (error) {
            console.error('Error loading keybindings:', error);
            keybindingsErrorMessage = 'Failed to load keybindings';
        } finally {
            isLoadingKeybindings = false;
        }
    }
    
    // Get the current keybinding for a command
    function getKeybinding(commandId: number): string {
        // First check if the user has a custom keybinding
        const customKeybinding = userKeybindings.find(kb => kb.command_id === commandId);
        if (customKeybinding) {
            return customKeybinding.keybinding;
        }
        
        // Otherwise, return the default keybinding
        const command = commands.find(cmd => cmd.command_id === commandId);
        return command ? command.default_keybinding : '';
    }
    
    // Start editing a keybinding
    function startEditKeybinding(commandId: number) {
        editingKeybinding = commandId;
        newKeybindingValue = getKeybinding(commandId);
    }
    
    // Cancel editing a keybinding
    function cancelEditKeybinding() {
        editingKeybinding = null;
        newKeybindingValue = '';
    }
    
    // Save a keybinding
    async function saveKeybinding(commandId: number) {
        try {
            keybindingsErrorMessage = '';
            keybindingsSuccessMessage = '';
            
            if (!newKeybindingValue.trim()) {
                keybindingsErrorMessage = 'Keybinding cannot be empty';
                return;
            }
            
            const result = await add_update_keybinding(commandId, newKeybindingValue);
            
            if (result) {
                // Update local state
                const existingIndex = userKeybindings.findIndex(kb => kb.command_id === commandId);
                if (existingIndex >= 0) {
                    userKeybindings[existingIndex] = result;
                } else {
                    userKeybindings = [...userKeybindings, result];
                }
                
                keybindingsSuccessMessage = 'Keybinding updated successfully';
                editingKeybinding = null;
                newKeybindingValue = '';
            } else {
                keybindingsErrorMessage = 'Failed to update keybinding';
            }
            
        } catch (error) {
            console.error('Error saving keybinding:', error);
            keybindingsErrorMessage = 'An unexpected error occurred';
        }
    }
    
    // Reset a keybinding to default
    async function resetKeybinding(commandId: number) {
        try {
            keybindingsErrorMessage = '';
            keybindingsSuccessMessage = '';
            
            const result = await delete_keybinding(commandId);
            
            if (result) {
                // Remove from user keybindings
                userKeybindings = userKeybindings.filter(kb => kb.command_id !== commandId);
                
                keybindingsSuccessMessage = 'Keybinding reset to default';
                
                // If we were editing this keybinding, clear the edit state
                if (editingKeybinding === commandId) {
                    editingKeybinding = null;
                    newKeybindingValue = '';
                }
            } else {
                keybindingsErrorMessage = 'Failed to reset keybinding';
            }
            
        } catch (error) {
            console.error('Error resetting keybinding:', error);
            keybindingsErrorMessage = 'An unexpected error occurred';
        }
    }
    
    // Check if a command has a custom keybinding
    function hasCustomKeybinding(commandId: number): boolean {
        return userKeybindings.some(kb => kb.command_id === commandId);
    }
    
    // Format the keybinding for display (e.g., "Ctrl+B" -> "Ctrl + B")
    function formatKeybinding(keybinding: string): string {
        return keybinding.split('+').join(' + ');
    }
    
    // Handle a keydown event in the keybinding input
    function handleKeybindingKeydown(event: KeyboardEvent) {
        event.preventDefault();
        
        const keys: string[] = [];
        
        if (event.ctrlKey) keys.push('Ctrl');
        if (event.shiftKey) keys.push('Shift');
        if (event.altKey) keys.push('Alt');
        if (event.metaKey) keys.push('Meta');
        
        // Add the key if it's not a modifier key
        if (!['Control', 'Shift', 'Alt', 'Meta'].includes(event.key)) {
            // Format the key nicely
            const key = event.key.length === 1 ? event.key.toUpperCase() : event.key;
            keys.push(key);
        }
        
        // Only set the value if there's at least one modifier and one regular key
        if (keys.length > 1) {
            newKeybindingValue = keys.join('+');
        }
    }
</script>

<svelte:head>
    <title>My Account | Vynn</title>
</svelte:head>

<div class="bg-black min-vh-100 d-flex flex-column">
    <Navbar {isLoggedIn} />
    
    <div class="container py-5">
        <div class="row justify-content-center">
            <div class="col-12 col-lg-8">
                <!-- Tabs navigation -->
                <ul class="nav nav-tabs mb-4">
                    <li class="nav-item">
                        <button 
                            class="nav-link text-white {activeTab === 'profile' ? 'active bg-dark' : ''}" 
                            on:click={() => activeTab = 'profile'}
                        >
                            <i class="bi bi-person me-2"></i> Profile
                        </button>
                    </li>
                    <li class="nav-item">
                        <button 
                            class="nav-link text-white {activeTab === 'keybindings' ? 'active bg-dark' : ''}" 
                            on:click={() => activeTab = 'keybindings'}
                        >
                            <i class="bi bi-keyboard me-2"></i> Keybindings
                        </button>
                    </li>
                </ul>
                
                {#if activeTab === 'profile'}
                    <!-- Profile Tab Content -->
                    <div class="card bg-dark text-white border-0 shadow">
                        <div class="card-body p-4">
                            <h2 class="card-title text-center mb-4">Profile Information</h2>
                            
                            {#if isLoading}
                                <div class="text-center p-4">
                                    <div class="spinner-border text-green" role="status">
                                        <span class="visually-hidden">Loading...</span>
                                    </div>
                                </div>
                            {:else}
                                <!-- Success message -->
                                {#if successMessage}
                                    <div class="alert alert-success mb-4" role="alert">
                                        {successMessage}
                                    </div>
                                {/if}
                                
                                <!-- Error message -->
                                {#if errorMessage}
                                    <div class="alert alert-danger mb-4" role="alert">
                                        {errorMessage}
                                    </div>
                                {/if}
                                
                                <form on:submit|preventDefault={handleSubmit}>
                                    <!-- Profile Image -->
                                    <div class="text-center mb-4">
                                        <div class="position-relative mx-auto" style="width: 150px; height: 150px;">
                                            <img 
                                                src={imagePreview || profileImage} 
                                                alt="Profile" 
                                                class="rounded-circle bg-black"
                                                style="width: 150px; height: 150px; object-fit: cover; border: 3px solid var(--color-primary);"
                                            />
                                            <label 
                                                for="profileImageInput" 
                                                class="position-absolute bottom-0 end-0 bg-dark rounded-circle p-2 cursor-pointer"
                                                style="cursor: pointer;"
                                            >
                                                <i class="bi bi-camera-fill text-green"></i>
                                                <span class="visually-hidden">Change profile picture</span>
                                            </label>
                                        </div>
                                        <input 
                                            type="file" 
                                            id="profileImageInput" 
                                            accept="image/*" 
                                            class="d-none"
                                            on:change={handleFileSelect}
                                        />
                                    </div>
                                    
                                    <!-- Name -->
                                    <div class="mb-3">
                                        <label for="name" class="form-label">Name</label>
                                        <input 
                                            type="text" 
                                            class="form-control bg-black text-white border-secondary" 
                                            id="name" 
                                            bind:value={name} 
                                            required
                                        />
                                    </div>
                                    
                                    <!-- Email -->
                                    <div class="mb-3">
                                        <label for="email" class="form-label">Email</label>
                                        <input 
                                            type="email" 
                                            class="form-control bg-black text-white border-secondary" 
                                            id="email" 
                                            bind:value={email} 
                                            required
                                        />
                                    </div>
                                    
                                    <!-- Password -->
                                    <div class="mb-3">
                                        <label for="password" class="form-label">New Password (leave blank to keep current)</label>
                                        <input 
                                            type="password" 
                                            class="form-control bg-black text-white border-secondary" 
                                            id="password" 
                                            bind:value={password} 
                                        />
                                    </div>
                                    
                                    <!-- Confirm Password -->
                                    <div class="mb-3">
                                        <label for="confirmPassword" class="form-label">Confirm New Password</label>
                                        <input 
                                            type="password" 
                                            class="form-control bg-black text-white border-secondary" 
                                            id="confirmPassword" 
                                            bind:value={confirmPassword} 
                                        />
                                    </div>
                                    
                                    <!-- Submit Button -->
                                    <button 
                                        type="submit" 
                                        class="btn btn-green w-100" 
                                        disabled={isSaving}
                                    >
                                        {#if isSaving}
                                            <span class="spinner-border spinner-border-sm me-2" role="status" aria-hidden="true"></span>
                                            Saving...
                                        {:else}
                                            Save Changes
                                        {/if}
                                    </button>
                                </form>
                            {/if}
                        </div>
                    </div>
                {:else if activeTab === 'keybindings'}
                    <!-- Keybindings Tab Content -->
                    <div class="card bg-dark text-white border-0 shadow">
                        <div class="card-body p-4">
                            <h2 class="card-title text-center mb-4">Customize Keybindings</h2>
                            
                            <p class="text-white-50 mb-4">
                                Customize keybindings for various commands to match your preferences. 
                                To change a keybinding, click the edit button, press the desired key combination, and save.
                            </p>
                            
                            {#if isLoadingKeybindings}
                                <div class="text-center p-4">
                                    <div class="spinner-border text-green" role="status">
                                        <span class="visually-hidden">Loading...</span>
                                    </div>
                                </div>
                            {:else}
                                <!-- Success message -->
                                {#if keybindingsSuccessMessage}
                                    <div class="alert alert-success mb-4 alert-dismissible fade show" role="alert">
                                        {keybindingsSuccessMessage}
                                        <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>
                                    </div>
                                {/if}
                                
                                <!-- Error message -->
                                {#if keybindingsErrorMessage}
                                    <div class="alert alert-danger mb-4 alert-dismissible fade show" role="alert">
                                        {keybindingsErrorMessage}
                                        <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>
                                    </div>
                                {/if}
                                
                                <!-- Keybindings table -->
                                <div class="table-responsive">
                                    <table class="table table-dark table-hover">
                                        <thead>
                                            <tr>
                                                <th>Command</th>
                                                <th>Description</th>
                                                <th>Keybinding</th>
                                                <th>Actions</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {#each commands as command}
                                                <tr>
                                                    <td>{command.command_name}</td>
                                                    <td>{command.command_description}</td>
                                                    <td>
                                                        {#if editingKeybinding === command.command_id}
                                                            <input 
                                                                type="text" 
                                                                class="form-control bg-black text-white border-secondary" 
                                                                bind:value={newKeybindingValue}
                                                                placeholder="Press keys..."
                                                                on:keydown={handleKeybindingKeydown}
                                                                readonly
                                                            />
                                                        {:else}
                                                            <span class="{hasCustomKeybinding(command.command_id) ? 'text-green' : ''}">
                                                                {formatKeybinding(getKeybinding(command.command_id))}
                                                            </span>
                                                            {#if hasCustomKeybinding(command.command_id)}
                                                                <span class="badge rounded-pill bg-green ms-2">Custom</span>
                                                            {/if}
                                                        {/if}
                                                    </td>
                                                    <td>
                                                        {#if editingKeybinding === command.command_id}
                                                            <div class="btn-group btn-group-sm">
                                                                <button 
                                                                    class="btn btn-success" 
                                                                    aria-label="Close"
                                                                    on:click={() => saveKeybinding(command.command_id)}
                                                                >
                                                                    <i class="bi bi-check"></i>
                                                                </button>
                                                                <button 
                                                                    class="btn btn-danger" 
                                                                    aria-label="Close"
                                                                    on:click={cancelEditKeybinding}
                                                                >
                                                                    <i class="bi bi-x"></i>
                                                                </button>
                                                            </div>
                                                        {:else}
                                                            <div class="btn-group btn-group-sm">
                                                                <button 
                                                                    class="btn btn-outline-light" 
                                                                    title="Edit keybinding"
                                                                    aria-label="Close"
                                                                    on:click={() => startEditKeybinding(command.command_id)}
                                                                >
                                                                    <i class="bi bi-pencil"></i>
                                                                </button>
                                                                {#if hasCustomKeybinding(command.command_id)}
                                                                    <button 
                                                                        class="btn btn-outline-danger" 
                                                                        title="Reset to default"
                                                                        aria-label="Close"
                                                                        on:click={() => resetKeybinding(command.command_id)}
                                                                    >
                                                                        <i class="bi bi-arrow-counterclockwise"></i>
                                                                    </button>
                                                                {/if}
                                                            </div>
                                                        {/if}
                                                    </td>
                                                </tr>
                                            {/each}
                                        </tbody>
                                    </table>
                                </div>
                            {/if}
                        </div>
                    </div>
                {/if}
                
                <!-- Account Links -->
                <div class="card bg-dark text-white border-0 shadow mt-4">
                    <div class="card-body p-4">
                        <h3 class="card-title mb-3">Account Management</h3>
                        
                        <div class="list-group list-group-flush bg-dark">
                            <a href="/drive" class="list-group-item list-group-item-action bg-dark text-white border-secondary">
                                <i class="bi bi-folder me-2"></i> My Documents & Projects
                            </a>
                            <a href="/document" class="list-group-item list-group-item-action bg-dark text-white border-secondary">
                                <i class="bi bi-file-earmark-text me-2"></i> Recent Documents
                            </a>
                            <button class="list-group-item list-group-item-action bg-dark text-danger border-secondary">
                                <i class="bi bi-trash me-2"></i> Delete Account
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>