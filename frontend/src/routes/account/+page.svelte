<script lang="ts">
    import { onMount } from 'svelte';
    import { get_current_user, update_user, upload_profile_image, get_profile_image_url } from '$lib/ts/user';
    import { get_all_commands, get_all_keybindings, add_update_keybinding, delete_keybinding, Command, UserKeybinding } from '$lib/ts/document';
    import Navbar from '$lib/components/Navbar.svelte';
    import profileDefault from '$lib/assets/profile-image.png';
    
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
    
    // Custom keybinding creation
    let showCustomKeybindingForm = false;
    let customCommandName = '';
    let customCommandDescription = '';
    let customKeybindingValue = '';
    let customCommandAction = 'toggleDarkMode'; // Default action
    let customKeybindings: Command[] = [];
    let nextCustomCommandId = -1; // Negative IDs for custom commands to avoid conflicts with predefined commands
    
    // Available actions for custom keybindings
    const availableActions = [
        { id: 'toggleDarkMode', name: 'Toggle Dark Mode', description: 'Switch between light and dark themes' },
        { id: 'saveAllDocuments', name: 'Save All Documents', description: 'Save all open documents' },
        { id: 'focusSearch', name: 'Focus Search', description: 'Move cursor to search box' },
        { id: 'toggleSplitView', name: 'Toggle Split View', description: 'Switch between single and split view layouts' },
        { id: 'openRecentDocuments', name: 'Show Recent Documents', description: 'Display a list of recently accessed documents' },
        { id: 'formatDocument', name: 'Format Document', description: 'Format the current document' },
        { id: 'findReplace', name: 'Find & Replace', description: 'Open the find and replace dialog' },
        { id: 'toggleSidebar', name: 'Toggle Sidebar', description: 'Show or hide the sidebar' },
        { id: 'newDocument', name: 'New Document', description: 'Create a new document' },
        { id: 'toggleFullscreen', name: 'Toggle Fullscreen', description: 'Enter or exit fullscreen mode' }
    ];
    
    // Suggested custom commands
    const suggestedCommands = [
        {
            name: 'Toggle Dark Mode',
            description: 'Switch between light and dark themes',
            keybinding: 'Ctrl+Shift+D',
            actionId: 'toggleDarkMode'
        },
        {
            name: 'Save All Open Documents',
            description: 'Save all currently open documents',
            keybinding: 'Ctrl+Alt+S',
            actionId: 'saveAllDocuments'
        },
        {
            name: 'Focus Search',
            description: 'Move cursor to search box',
            keybinding: 'Ctrl+Shift+F',
            actionId: 'focusSearch'
        },
        {
            name: 'Split View',
            description: 'Toggle split view for side-by-side editing',
            keybinding: 'Ctrl+Alt+V',
            actionId: 'toggleSplitView'
        },
        {
            name: 'Open Recent Documents',
            description: 'Show a list of recently accessed documents',
            keybinding: 'Ctrl+Shift+R',
            actionId: 'openRecentDocuments'
        }
    ];
    
    // Metadata for custom keybindings (not stored in Command object)
    let customKeybindingsMetadata: Map<number, { actionId: string }> = new Map();
    
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
    
    // Handle creating a new custom keybinding
    function createCustomKeybinding() {
        if (!customCommandName.trim()) {
            keybindingsErrorMessage = 'Command name cannot be empty';
            return;
        }
        
        if (!customCommandDescription.trim()) {
            keybindingsErrorMessage = 'Command description cannot be empty';
            return;
        }
        
        if (!customKeybindingValue.trim()) {
            keybindingsErrorMessage = 'Keybinding cannot be empty';
            return;
        }
        
        if (!customCommandAction) {
            keybindingsErrorMessage = 'Please select an action for this keybinding';
            return;
        }
        
        // Create new custom command
        const cmdId = nextCustomCommandId;
        const newCommand = new Command(
            cmdId,
            customCommandName,
            customCommandDescription,
            customKeybindingValue
        );
        
        // Add to local custom commands list
        customKeybindings = [...customKeybindings, newCommand];
        
        // Store the action metadata
        customKeybindingsMetadata.set(cmdId, { actionId: customCommandAction });
        
        // Create a user keybinding entry
        const newKeybinding = new UserKeybinding(
            userId,
            cmdId,
            customKeybindingValue
        );
        
        // Add to user keybindings
        userKeybindings = [...userKeybindings, newKeybinding];
        
        // Decrement the next custom command ID (to keep them negative)
        nextCustomCommandId--;
        
        // Reset form
        customCommandName = '';
        customCommandDescription = '';
        customKeybindingValue = '';
        customCommandAction = 'toggleDarkMode'; // Reset to default action
        showCustomKeybindingForm = false;
        
        keybindingsSuccessMessage = 'Custom keybinding created successfully';
    }
    
    // Handle custom keybinding key input
    function handleCustomKeybindingKeydown(event: KeyboardEvent) {
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
            customKeybindingValue = keys.join('+');
        }
    }
    
    // Get all commands (predefined + custom)
    function getAllCommands() {
        return [...commands, ...customKeybindings];
    }
    
    // Use a suggested command as a starting point for a custom command
    function useSuggestedCommand(command: { name: string, description: string, keybinding: string, actionId: string }) {
        console.log("Adding suggested command:", command);
        
        // Check if a command with the same name already exists
        const existingCommand = [...commands, ...customKeybindings].find(
            cmd => cmd.command_name.toLowerCase() === command.name.toLowerCase()
        );
        
        if (existingCommand) {
            keybindingsErrorMessage = `A command named "${command.name}" already exists. Please choose a different name.`;
            
            // Scroll to the existing command to show the user
            setTimeout(() => {
                const existingRow = document.querySelector(`.keybinding-row[data-command-id="${existingCommand.command_id}"]`);
                if (existingRow) {
                    existingRow.classList.add('highlight-new-row');
                    existingRow.scrollIntoView({ behavior: 'smooth', block: 'center' });
                    
                    // Remove highlight after animation
                    setTimeout(() => {
                        existingRow.classList.remove('highlight-new-row');
                    }, 2500);
                }
            }, 200);
            
            return;
        }
        
        // Check if the keybinding is already in use
        const existingKeybinding = userKeybindings.find(
            kb => kb.keybinding.toLowerCase() === command.keybinding.toLowerCase()
        );
        
        if (existingKeybinding) {
            // Find the command that uses this keybinding
            const conflictingCommand = [...commands, ...customKeybindings].find(
                cmd => cmd.command_id === existingKeybinding.command_id
            );
            
            if (conflictingCommand) {
                keybindingsErrorMessage = `The keybinding "${command.keybinding}" is already used by "${conflictingCommand.command_name}". Please use a different keybinding.`;
                
                // Scroll to the conflicting command
                setTimeout(() => {
                    const conflictingRow = document.querySelector(`.keybinding-row[data-command-id="${conflictingCommand.command_id}"]`);
                    if (conflictingRow) {
                        conflictingRow.classList.add('highlight-new-row');
                        conflictingRow.scrollIntoView({ behavior: 'smooth', block: 'center' });
                        
                        // Remove highlight after animation
                        setTimeout(() => {
                            conflictingRow.classList.remove('highlight-new-row');
                        }, 2500);
                    }
                }, 200);
                
                return;
            }
        }
        
        // Automatically create the custom keybinding
        
        // Create new custom command with the next negative ID
        const cmdId = nextCustomCommandId;
        
        // Create new custom command
        const newCommand = new Command(
            cmdId,
            command.name,
            command.description,
            command.keybinding
        );
        
        console.log("Created new custom command:", newCommand);
        
        // Add to local custom commands list first
        customKeybindings = [...customKeybindings, newCommand];
        
        // Store the action metadata
        customKeybindingsMetadata.set(cmdId, { actionId: command.actionId });
        
        // Create a user keybinding entry
        const newKeybinding = new UserKeybinding(
            userId,
            cmdId,
            command.keybinding
        );
        
        console.log("Created new keybinding:", newKeybinding);
        
        // Add to user keybindings
        userKeybindings = [...userKeybindings, newKeybinding];
        
        // Decrement the next custom command ID (to keep them negative)
        nextCustomCommandId--;
        
        console.log("Current custom keybindings:", customKeybindings);
        
        // Show success message
        keybindingsSuccessMessage = `Custom keybinding "${command.name}" created successfully`;
        
        // Scroll to the newly added keybinding in the table after a short delay
        // to allow the DOM to update
        setTimeout(() => {
            // Find the newly added custom keybinding row specifically
            const customRow = document.querySelector(`.keybinding-row[data-command-id="${cmdId}"]`);
            
            if (customRow) {
                console.log("Found custom keybinding row, highlighting");
                // Add highlight class to the newly added row
                customRow.classList.add('highlight-new-row');
                
                // Scroll the table to show the new row
                customRow.scrollIntoView({ behavior: 'smooth', block: 'center' });
                
                // Remove the highlight class after animation completes
                setTimeout(() => {
                    customRow.classList.remove('highlight-new-row');
                }, 2500);
            } else {
                console.log("Custom keybinding row not found in DOM");
                
                // As a fallback, try to scroll to the custom keybindings section
                const customSection = document.querySelector('.text-green');
                if (customSection) {
                    customSection.scrollIntoView({ behavior: 'smooth', block: 'start' });
                }
            }
        }, 300);
    }
    
    // Get the action name for a custom keybinding
    function getActionForCommand(commandId: number): string {
        const metadata = customKeybindingsMetadata.get(commandId);
        if (metadata) {
            const action = availableActions.find(a => a.id === metadata.actionId);
            return action ? action.name : 'Unknown Action';
        }
        return 'Unknown Action';
    }
    
    // Get the action ID for a custom keybinding
    function getActionIdForCommand(commandId: number): string {
        const metadata = customKeybindingsMetadata.get(commandId);
        return metadata ? metadata.actionId : 'unknown';
    }
    
    // Handle changing a custom keybinding's action
    function updateCustomKeybindingAction(commandId: number, actionId: string) {
        customKeybindingsMetadata.set(commandId, { actionId });
        keybindingsSuccessMessage = 'Keybinding action updated successfully';
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
                                You can also create your own custom keybindings.
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
                                
                                <!-- Add Keybinding Button -->
                                <div class="mb-4">
                                    <button 
                                        class="btn btn-green" 
                                        on:click={() => showCustomKeybindingForm = !showCustomKeybindingForm}
                                    >
                                        <i class="bi bi-plus-circle me-2"></i>
                                        {showCustomKeybindingForm ? 'Cancel' : 'Add Custom Keybinding'}
                                    </button>
                                </div>
                                
                                <!-- Suggested Commands -->
                                <div class="mb-4 suggested-commands">
                                    <h5 class="mb-3">Suggested Commands</h5>
                                    <div class="row row-cols-1 row-cols-md-2 g-3">
                                        {#each suggestedCommands as command}
                                            <div class="col">
                                                <div class="card bg-black border-secondary h-100">
                                                    <div class="card-body">
                                                        <h6 class="card-title d-flex align-items-center">
                                                            {command.name}
                                                        </h6>
                                                        <p class="card-text small text-white-50">{command.description}</p>
                                                        <div class="d-flex align-items-center justify-content-between">
                                                            <span class="badge bg-dark border border-secondary">{command.keybinding}</span>
                                                            <button 
                                                                class="btn btn-sm btn-outline-success" 
                                                                on:click={() => useSuggestedCommand(command)}
                                                                title="Add this keybinding to your list"
                                                            >
                                                                <i class="bi bi-plus-circle me-1"></i> Add
                                                            </button>
                                                        </div>
                                                        <div class="mt-2 small text-muted">
                                                            Action: {command.actionId.replace(/([A-Z])/g, ' $1').trim()}
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        {/each}
                                    </div>
                                    <div class="text-muted text-center small mt-2">
                                        <i class="bi bi-info-circle me-1"></i> Click "Add" to instantly add a keybinding to your list below
                                    </div>
                                </div>
                                
                                <!-- Custom Keybinding Form -->
                                {#if showCustomKeybindingForm}
                                    <div id="customKeybindingForm" class="card bg-black border-secondary mb-4">
                                        <div class="card-body p-3">
                                            <h5 class="card-title mb-3">Create Custom Keybinding</h5>
                                            
                                            <div class="mb-3">
                                                <label for="customCommandName" class="form-label">Command Name</label>
                                                <input 
                                                    type="text" 
                                                    class="form-control bg-dark text-white border-secondary" 
                                                    id="customCommandName" 
                                                    bind:value={customCommandName}
                                                    placeholder="e.g., Toggle Dark Mode"
                                                    required
                                                />
                                            </div>
                                            
                                            <div class="mb-3">
                                                <label for="customCommandDescription" class="form-label">Command Description</label>
                                                <input 
                                                    type="text" 
                                                    class="form-control bg-dark text-white border-secondary" 
                                                    id="customCommandDescription" 
                                                    bind:value={customCommandDescription}
                                                    placeholder="e.g., Switch between light and dark themes"
                                                    required
                                                />
                                            </div>
                                            
                                            <div class="mb-3">
                                                <label for="customCommandAction" class="form-label">Action</label>
                                                <select 
                                                    class="form-select bg-dark text-white border-secondary" 
                                                    id="customCommandAction" 
                                                    bind:value={customCommandAction}
                                                    required
                                                >
                                                    {#each availableActions as action}
                                                        <option value={action.id}>{action.name} - {action.description}</option>
                                                    {/each}
                                                </select>
                                                <small class="form-text text-muted">Select what this keybinding will do when activated</small>
                                            </div>
                                            
                                            <div class="mb-3">
                                                <label for="customKeybindingValue" class="form-label">Keybinding</label>
                                                <input 
                                                    type="text" 
                                                    class="form-control bg-dark text-white border-secondary" 
                                                    id="customKeybindingValue" 
                                                    bind:value={customKeybindingValue}
                                                    placeholder="Press keys..."
                                                    on:keydown={handleCustomKeybindingKeydown}
                                                    readonly
                                                    required
                                                />
                                                <small class="form-text text-muted">Press a key combination (e.g., Ctrl+Shift+D)</small>
                                            </div>
                                            
                                            <button 
                                                class="btn btn-green" 
                                                on:click={createCustomKeybinding}
                                            >
                                                <i class="bi bi-save me-2"></i>
                                                Create Keybinding
                                            </button>
                                        </div>
                                    </div>
                                {/if}
                                
                                <!-- Keybindings table with max height for scrolling -->
                                <div class="table-responsive keybindings-table" style="max-height: 500px; overflow-y: auto;">
                                    <!-- Debug counts to ensure commands are being added -->
                                    <div class="text-muted mb-2 small">
                                        Total commands: {commands.length + customKeybindings.length} 
                                        (System: {commands.length}, Custom: {customKeybindings.length})
                                    </div>
                                    
                                    <table class="table table-dark table-hover">
                                        <thead style="position: sticky; top: 0; z-index: 1;">
                                            <tr class="bg-dark">
                                                <th>Command</th>
                                                <th>Description</th>
                                                <th>Keybinding</th>
                                                <th>Action</th>
                                                <th>Controls</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {#if customKeybindings.length > 0}
                                                <!-- Custom keybindings section -->
                                                <tr class="border-bottom border-secondary">
                                                    <td colspan="5" class="text-center bg-dark">
                                                        <strong class="text-green">Custom Keybindings</strong>
                                                    </td>
                                                </tr>
                                                
                                                {#each customKeybindings as command}
                                                    <tr class="keybinding-row custom-keybinding" data-command-id={command.command_id}>
                                                        <td>
                                                            {command.command_name}
                                                            <span class="badge rounded-pill bg-primary ms-2">Custom</span>
                                                        </td>
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
                                                                <span class="text-green">
                                                                    {formatKeybinding(command.default_keybinding)}
                                                                </span>
                                                            {/if}
                                                        </td>
                                                        <td>
                                                            <select 
                                                                class="form-select form-select-sm bg-dark text-white border-secondary" 
                                                                value={getActionIdForCommand(command.command_id)}
                                                                on:change={(e) => updateCustomKeybindingAction(command.command_id, e.currentTarget.value)}
                                                            >
                                                                {#each availableActions as action}
                                                                    <option value={action.id}>{action.name}</option>
                                                                {/each}
                                                            </select>
                                                        </td>
                                                        <td>
                                                            {#if editingKeybinding === command.command_id}
                                                                <div class="btn-group btn-group-sm">
                                                                    <button 
                                                                        class="btn btn-success" 
                                                                        on:click={() => saveKeybinding(command.command_id)}
                                                                        aria-label="Save keybinding"
                                                                    >
                                                                        <i class="bi bi-check"></i>
                                                                    </button>
                                                                    <button 
                                                                        class="btn btn-danger" 
                                                                        on:click={cancelEditKeybinding}
                                                                        aria-label="Cancel editing"
                                                                    >
                                                                        <i class="bi bi-x"></i>
                                                                    </button>
                                                                </div>
                                                            {:else}
                                                                <div class="btn-group btn-group-sm">
                                                                    <button 
                                                                        class="btn btn-outline-light" 
                                                                        title="Edit keybinding"
                                                                        on:click={() => startEditKeybinding(command.command_id)}
                                                                        aria-label="Edit binding"
                                                                    >
                                                                        <i class="bi bi-pencil"></i>
                                                                    </button>
                                                                    <button 
                                                                        class="btn btn-outline-danger" 
                                                                        title="Remove custom keybinding"
                                                                        on:click={() => resetKeybinding(command.command_id)}
                                                                        aria-label="Reset binding"
                                                                    >
                                                                        <i class="bi bi-trash"></i>
                                                                    </button>
                                                                </div>
                                                            {/if}
                                                        </td>
                                                    </tr>
                                                {/each}
                                                
                                                <!-- System keybindings separator -->
                                                <tr class="border-bottom border-secondary">
                                                    <td colspan="5" class="text-center bg-dark">
                                                        <strong>System Keybindings</strong>
                                                    </td>
                                                </tr>
                                            {/if}
                                            
                                            {#each commands as command}
                                                <tr class="keybinding-row" data-command-id={command.command_id}>
                                                    <td>
                                                        {command.command_name}
                                                    </td>
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
                                                        <span class="text-muted small">System action</span>
                                                    </td>
                                                    <td>
                                                        {#if editingKeybinding === command.command_id}
                                                            <div class="btn-group btn-group-sm">
                                                                <button 
                                                                    class="btn btn-success" 
                                                                    aria-label="Save keybinding"
                                                                    on:click={() => saveKeybinding(command.command_id)}
                                                                >
                                                                    <i class="bi bi-check"></i>
                                                                </button>
                                                                <button 
                                                                    class="btn btn-danger" 
                                                                    aria-label="Cancel editing"
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
                                                                    aria-label="Edit keybinding"
                                                                    on:click={() => startEditKeybinding(command.command_id)}
                                                                >
                                                                    <i class="bi bi-pencil"></i>
                                                                </button>
                                                                {#if hasCustomKeybinding(command.command_id)}
                                                                    <button 
                                                                        class="btn btn-outline-danger" 
                                                                        title="Reset to default"
                                                                        aria-label="Reset to default"
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

<style>
    :global(:root) {
        --color-primary: #10b981;
        --color-primary-dark: #059669;
        --color-primary-light: #34d399;
        --color-background: #121212;
        --color-background-rgb: 18, 18, 18;
    }
    
    :global(.btn-green) {
        background-color: var(--color-primary);
        border-color: var(--color-primary);
        color: white;
    }
    
    :global(.btn-green:hover) {
        background-color: var(--color-primary-dark);
        border-color: var(--color-primary-dark);
        color: white;
    }
    
    :global(.text-green) {
        color: var(--color-primary) !important;
    }
    
    .list-group-item:hover {
        background-color: rgba(255, 255, 255, 0.05) !important;
    }
    
    /* Nav tabs styling */
    .nav-tabs {
        border-bottom-color: #343a40;
    }
    
    .nav-tabs .nav-link {
        border: 1px solid transparent;
        border-top-left-radius: 0.25rem;
        border-top-right-radius: 0.25rem;
        color: #6c757d;
    }
    
    .nav-tabs .nav-link:hover {
        border-color: #343a40 #343a40 #343a40;
        color: var(--color-primary);
    }
    
    .nav-tabs .nav-link.active {
        color: var(--color-primary);
        background-color: #212529;
        border-color: #343a40 #343a40 #212529;
    }
    
    /* Badge styling */
    .bg-green {
        background-color: var(--color-primary) !important;
    }
    
    /* Table styling */
    .table-responsive {
        scrollbar-width: thin;
        scrollbar-color: var(--color-primary) #343a40;
    }
    
    .table-responsive::-webkit-scrollbar {
        width: 8px;
        height: 8px;
    }
    
    .table-responsive::-webkit-scrollbar-track {
        background: #343a40;
        border-radius: 4px;
    }
    
    .table-responsive::-webkit-scrollbar-thumb {
        background-color: var(--color-primary);
        border-radius: 4px;
    }
    
    /* Custom keybinding form */
    .card.bg-black {
        background-color: #121212;
    }
    
    /* Suggested commands styling */
    .suggested-commands .card {
        transition: transform 0.2s, box-shadow 0.2s;
        height: 100%;
    }
    
    .suggested-commands .card:hover {
        transform: translateY(-2px);
        box-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
        border-color: var(--color-primary) !important;
    }
    
    .suggested-commands .card-title {
        font-weight: 600;
        color: var(--color-primary-light);
    }
    
    .suggested-commands .badge {
        font-family: monospace;
        letter-spacing: 0.5px;
        background-color: #1a1a1a;
        padding: 6px 8px;
    }
    
    .suggested-commands .btn-outline-success {
        border-color: var(--color-primary);
        color: var(--color-primary);
    }
    
    .suggested-commands .btn-outline-success:hover {
        background-color: var(--color-primary);
        color: white;
        border-color: var(--color-primary);
    }
    
    /* Keybindings table styling */
    .keybindings-table tr.keybinding-row {
        transition: background-color 0.3s ease;
    }
    
    .keybindings-table tr[data-command-id^="-"] td:first-child {
        border-left: 3px solid var(--color-primary);
    }
    
    /* Custom keybindings styling */
    .keybindings-table tr.custom-keybinding {
        background-color: rgba(16, 185, 129, 0.05);
    }
    
    .keybindings-table tr.custom-keybinding:hover {
        background-color: rgba(16, 185, 129, 0.1);
    }
    
    .keybindings-table .custom-keybinding td:first-child {
        border-left: 3px solid var(--color-primary);
    }
    
    /* Section headers in table */
    .keybindings-table tr.border-bottom.border-secondary td {
        padding: 10px;
        font-size: 0.9rem;
        letter-spacing: 0.5px;
        text-transform: uppercase;
    }
    
    /* Highlight newly added keybinding */
    @keyframes highlight-row {
        0% { background-color: rgba(16, 185, 129, 0.3); }
        50% { background-color: rgba(16, 185, 129, 0.2); }
        100% { background-color: transparent; }
    }
</style>