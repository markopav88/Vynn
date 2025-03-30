<script lang="ts">
    import { onMount } from 'svelte';
    import { attempt_signup, check_auth } from "$lib/ts/user";
    import logo from '$lib/assets/logo.png';
    import Navbar from '$lib/components/Navbar.svelte';
    
    let firstName = '';
    let lastName = '';
    let email = '';
    let password = '';
    let confirmPassword = '';
    let agreeToTerms = false;
    let isLoggedIn = false;
    let showPassword = false;
    let passwordStrength = 0;
    let isLoading = false;
    let errorMessage = '';
    let pageLoaded = false;
    
    onMount(async () => {
        // Check authentication status using backend API
        const isAuthenticated = await check_auth();
        
        if (isAuthenticated) {
            console.log("User is authenticated, redirecting to /drive");
            window.location.href = '/drive';
            return;
        }
        
        // Trigger fade-in animation after component mounts
        setTimeout(() => {
            pageLoaded = true;
        }, 100);
    });
    
    // Check password strength
    function checkPasswordStrength(pass: string) {
        let score = 0;
        
        // Length check
        if (pass.length >= 8) score += 25;
        
        // Contains uppercase
        if (/[A-Z]/.test(pass)) score += 25;
        
        // Contains number
        if (/[0-9]/.test(pass)) score += 25;
        
        // Contains special character
        if (/[^A-Za-z0-9]/.test(pass)) score += 25;
        
        passwordStrength = score;
    }
    
    // Toggle password visibility
    function togglePasswordVisibility() {
        showPassword = !showPassword;
    }
    
    // Update password strength when password changes
    $: checkPasswordStrength(password);
    
    async function handleSubmit() {
        try {
            isLoading = true;
            errorMessage = '';
            
            // Verify passwords match before submission
            if (password !== confirmPassword) {
                errorMessage = 'Passwords do not match';
                return;
            }
            
            // Call the attempt_signup function with the correct parameter structure
            const success = await attempt_signup({
                name: `${firstName} ${lastName}`,
                email,
                password_one: password,
                password_two: confirmPassword
            });
            
            // Check the boolean return value
            if (success) {
                // If successful, redirect to login
                window.location.href = '/login?registered=true';
            } else {
                // If the function returns false but doesn't throw an error
                errorMessage = 'Account creation failed. Please try again.';
            }
            
        } catch (error: any) {
            console.error('Account creation error:', error);
            errorMessage = error.message || 'Failed to create account. Please try again.';
        } finally {
            isLoading = false;
        }
    }
</script>

<div class="bg-black min-vh-100 d-flex flex-column">
    <Navbar {isLoggedIn} />
    
    <div class="container flex-grow-1 d-flex align-items-center justify-content-center py-5 my-5">
        <div class="card bg-dark text-white border-0 shadow-lg fade-in {pageLoaded ? 'visible' : ''}" style="max-width: 450px; width: 100%;">
            <div class="card-body p-4 p-md-5">
                <!-- Logo and Title -->
                <div class="text-center mb-4 fade-element">
                    <img src={logo} alt="Vynn Logo" height="50" width="50" class="mb-3" />
                    <h2 class="fw-bold">Create your account</h2>
                    <p class="text-white-50">Join writers worldwide using Vynn</p>
                </div>
                
                <!-- Error Message -->
                {#if errorMessage}
                    <div class="alert alert-danger mb-4" role="alert">
                        {errorMessage}
                    </div>
                {/if}
                
                <!-- Signup Form -->
                <form on:submit|preventDefault={handleSubmit} class="fade-element">
                    <!-- Name Fields -->
                    <div class="row g-3 mb-3">
                        <div class="col-md-6">
                            <label for="firstName" class="form-label small text-white-50">First name</label>
                            <input 
                                type="text" 
                                class="form-control bg-black text-white border-dark" 
                                id="firstName" 
                                bind:value={firstName}
                                required
                            />
                        </div>
                        <div class="col-md-6">
                            <label for="lastName" class="form-label small text-white-50">Last name</label>
                            <input 
                                type="text" 
                                class="form-control bg-black text-white border-dark" 
                                id="lastName" 
                                bind:value={lastName}
                                required
                            />
                        </div>
                    </div>
                    
                    <!-- Email -->
                    <div class="mb-3">
                        <label for="email" class="form-label small text-white-50">Email</label>
                        <input 
                            type="email" 
                            class="form-control bg-black text-white border-dark" 
                            id="email" 
                            bind:value={email}
                            placeholder="your@example.com"
                            required
                        />
                    </div>
                    
                    <!-- Password -->
                    <div class="mb-3">
                        <label for="password" class="form-label small text-white-50">Password</label>
                        <div class="input-group">
                            <input 
                                type={showPassword ? "text" : "password"}
                                class="form-control bg-black text-white border-dark" 
                                id="password" 
                                bind:value={password}
                                placeholder="8+ characters"
                                required
                                minlength="8"
                            />
                            <button 
                                type="button"
                                class="input-group-text bg-black border-dark text-white-50"
                                on:click={togglePasswordVisibility}
                                aria-label="Toggle password visibility"
                            >
                                <i class="bi {showPassword ? 'bi-eye-slash' : 'bi-eye'}"></i>
                            </button>
                        </div>
                        <div class="progress mt-2" style="height: 5px;">
                            <div class="progress-bar bg-green" style="width: {passwordStrength}%"></div>
                        </div>
                        <div class="mt-1 d-flex justify-content-between">
                            <small class="text-white-50">Strength:</small>
                            <small class="{passwordStrength < 50 ? 'text-danger' : passwordStrength < 100 ? 'text-warning' : 'text-success'}">
                                {#if passwordStrength < 25}
                                    Very Weak
                                {:else if passwordStrength < 50}
                                    Weak
                                {:else if passwordStrength < 75}
                                    Good
                                {:else if passwordStrength < 100}
                                    Strong
                                {:else}
                                    Very Strong
                                {/if}
                            </small>
                        </div>
                    </div>
                    
                    <!-- Confirm Password -->
                    <div class="mb-4">
                        <label for="confirmPassword" class="form-label small text-white-50">Confirm Password</label>
                        <div class="input-group">
                            <input 
                                type={showPassword ? "text" : "password"}
                                class="form-control bg-black text-white border-dark" 
                                id="confirmPassword" 
                                bind:value={confirmPassword}
                                required
                            />
                        </div>
                        {#if confirmPassword && password !== confirmPassword}
                            <small class="text-danger">Passwords do not match</small>
                        {/if}
                    </div>
                    
                    <!-- Terms and Conditions -->
                    <div class="mb-4">
                        <div class="form-check">
                            <input 
                                class="form-check-input" 
                                type="checkbox" 
                                id="agreeTerms" 
                                bind:checked={agreeToTerms}
                                required
                            />
                            <label class="form-check-label small text-white-50" for="agreeTerms">
                                I agree to the <a href="/terms" class="text-green">Terms of Service</a> and <a href="/privacy" class="text-green">Privacy Policy</a>
                            </label>
                        </div>
                    </div>
                    
                    <!-- Submit Button -->
                    <button 
                        type="submit" 
                        class="btn btn-green w-100 py-2 mb-3"
                        disabled={isLoading || !agreeToTerms || password !== confirmPassword || passwordStrength < 50}
                    >
                        {#if isLoading}
                            <span class="spinner-border spinner-border-sm me-2" role="status" aria-hidden="true"></span>
                            Creating account...
                        {:else}
                            Create account
                        {/if}
                    </button>
                    
                    <!-- Login Link -->
                    <div class="text-center">
                        <span class="text-white-50 small">Already have an account?</span>
                        <a href="/login" class="text-green small ms-1">Sign in</a>
                    </div>
                </form>
                
                <!-- Social Links -->
                <div class="d-flex justify-content-center gap-3 mt-4 fade-element">
                    <button class="btn btn-link text-white-50 p-0 border-0" aria-label="GitHub">
                        <i class="bi bi-github"></i>
                    </button>
                    <button class="btn btn-link text-white-50 p-0 border-0" aria-label="Google">
                        <i class="bi bi-google"></i>
                    </button>
                    <button class="btn btn-link text-white-50 p-0 border-0" aria-label="Twitter">
                        <i class="bi bi-twitter"></i>
                    </button>
                </div>
            </div>
        </div>
    </div>
</div>

<style>
    /* Custom styles for the signup page */
    .form-control:focus, .form-check-input:focus {
        border-color: var(--color-primary);
        box-shadow: 0 0 0 0.25rem rgba(16, 185, 129, 0.25);
    }
    
    .form-check-input:checked {
        background-color: var(--color-primary);
        border-color: var(--color-primary);
    }
    
    .card {
        background: linear-gradient(145deg, #0a0a0a, #1a1a1a);
        border-radius: 12px;
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    }
    
    /* Green glow effect on the card */
    .card::before {
        content: '';
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        border-radius: 12px;
        padding: 1px;
        background: linear-gradient(145deg, rgba(16, 185, 129, 0.3), transparent);
        -webkit-mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
        mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
        -webkit-mask-composite: xor;
        mask-composite: exclude;
        pointer-events: none;
    }
    
    /* Ensure the navbar is consistent with landing page */
    :global(nav.navbar .navbar-collapse) {
        justify-content: center !important;
        flex-grow: 1 !important;
    }
    
    :global(nav.navbar .navbar-nav) {
        margin-left: auto !important;
        margin-right: auto !important;
    }
    
    /* Fade-in animations */
    .fade-in {
        opacity: 0;
        transform: translateY(20px);
        transition: opacity 0.8s ease, transform 0.8s ease;
    }
    
    .fade-in.visible {
        opacity: 1;
        transform: translateY(0);
    }
    
    .fade-element {
        opacity: 0;
        transform: translateY(15px);
        animation: fadeIn 0.8s ease forwards;
        animation-delay: 0.3s;
    }
    
    @keyframes fadeIn {
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }
    
    /* Staggered delays for fade elements */
    .fade-element:nth-child(1) { animation-delay: 0.3s; }
    .fade-element:nth-child(2) { animation-delay: 0.5s; }
    .fade-element:nth-child(3) { animation-delay: 0.7s; }
</style> 