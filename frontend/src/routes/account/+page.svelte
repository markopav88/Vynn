<script lang="ts">
    import { onMount } from 'svelte';
    import { get_current_user, update_user, upload_profile_image, get_profile_image_url } from '$lib/ts/user';
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
</script>

<svelte:head>
    <title>My Account | Vynn</title>
</svelte:head>

<div class="bg-black min-vh-100 d-flex flex-column">
    <Navbar {isLoggedIn} />
    
    <div class="container py-5">
        <div class="row justify-content-center">
            <div class="col-12 col-md-8 col-lg-6">
                <div class="card bg-dark text-white border-0 shadow">
                    <div class="card-body p-4">
                        <h2 class="card-title text-center mb-4">My Account</h2>
                        
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
</style> 