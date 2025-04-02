<script lang="ts">
    import { onMount } from 'svelte';
    import { logout } from '$lib/ts/user';
    import logo from '$lib/assets/logo.png';
    import profileDefault from '$lib/assets/profile-image.png';
    
    export let isLoggedIn = false;
    
    async function handleLogout() {
        try {
            console.log("Attempting to logout...");
            const success = await logout();
            
            if (success) {
                console.log("Logout successful, redirecting to homepage");
                window.location.href = '/';
            } else {
                console.error("Logout failed");
                alert("Failed to logout. Please try again.");
            }
        } catch (error) {
            console.error("Error during logout:", error);
        }
    }
    
    // Add handler for account page navigation
    function goToAccount() {
        window.location.href = '/account';
    }
</script>

<nav class="navbar navbar-expand navbar-dark bg-black">
    <div class="container-fluid">
        <!-- Logo and Brand Name -->
        <a class="navbar-brand d-flex align-items-center" href="/">
            <img src={logo} alt="Vynn Logo" class="me-2" height="60" width="60" />
            <span class="text-white fw-semibold">Vynn</span>
        </a>
        
        <!-- Navigation Links - Always Centered -->
        <div class="navbar-collapse justify-content-center flex-grow-1 mr-5">
            <ul class="navbar-nav mx-auto text-center">
                <li class="nav-item mx-3">
                    <a class="nav-link" href="/drive">My Drive</a>
                </li>
                <li class="nav-item mx-3">
                    <a class="nav-link" href="/document">Editor</a>
                </li>
                <li class="nav-item mx-3">
                    <a class="nav-link" href="/tutorial">Tutorial</a>
                </li>
                <li class="nav-item mx-3">
                    <a class="nav-link" href="/help">Help</a>
                </li>
            </ul>
        </div>
        
        <!-- Action Buttons -->
        <div>
            {#if isLoggedIn}
                <div class="dropdown">
                    <button 
                        class="btn p-0 border-0 bg-transparent" 
                        data-bs-toggle="dropdown"
                        aria-expanded="false"
                        aria-haspopup="true"
                        aria-label="Profile menu"
                    >
                        <img 
                            src={profileDefault} 
                            alt="Profile" 
                            class="rounded-circle profile-img"
                            style="width: 40px; height: 40px; border: 2px solid var(--color-primary);"
                        />
                    </button>
                    <ul class="dropdown-menu dropdown-menu-end dropdown-menu-dark">
                        <li>
                            <button class="dropdown-item" on:click={goToAccount}>
                                <i class="bi bi-person me-2"></i> My Account
                            </button>
                        </li>
                        <li><hr class="dropdown-divider"></li>
                        <li>
                            <button class="dropdown-item text-danger" on:click={handleLogout}>
                                <i class="bi bi-box-arrow-right me-2"></i> Sign Out
                            </button>
                        </li>
                    </ul>
                </div>
            {:else}
                <a 
                    href="/signup" 
                    class="btn btn-green btn-sm rounded-pill"
                >
                    Get Started
                </a>
            {/if}
        </div>
    </div>
</nav>

<style>
    nav {
        border-bottom: none;
    }
    
    .navbar-brand span {
        letter-spacing: 0.05em;
        font-size: 1.5rem;
    }
    
    /* Ensure navbar stays horizontal on all screen sizes */
    @media (max-width: 992px) {
        .navbar-collapse {
            display: flex !important;
        }
        
        .navbar-nav {
            flex-direction: row !important;
        }
        
        .nav-item {
            white-space: nowrap;
        }
    }
    
    /* Adjust spacing on very small screens */
    @media (max-width: 576px) {
        .nav-item {
            margin-left: 0.5rem !important;
            margin-right: 0.5rem !important;
        }
        
        .navbar-brand img {
            height: 40px;
            width: 40px;
        }
        
        .navbar-brand span {
            font-size: 1.2rem;
        }
    }
    
    /* Profile image hover effect */
    .profile-img {
        transition: all 0.2s ease;
    }
    
    .profile-img:hover {
        transform: scale(1.05);
        box-shadow: 0 0 10px rgba(16, 185, 129, 0.5);
    }
    
    /* Add these styles to your Navbar component */
    .nav-link {
        color: rgba(255, 255, 255, 0.8) !important;
        transition: color 0.2s ease;
        cursor: pointer;
    }
    
    .nav-link:hover {
        color: var(--color-primary) !important;
    }
    
    /* Prevent any hover-based navigation */
    a {
        pointer-events: auto;
    }
    
    a:hover {
        text-decoration: none;
    }
</style> 