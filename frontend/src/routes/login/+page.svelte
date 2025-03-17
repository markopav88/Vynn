<script lang="ts">
    import { onMount } from 'svelte';
    import { attempt_login, Login } from "$lib/ts/login"
	import { error } from '@sveltejs/kit';
    
    let email = '';
    let password = '';
    let errorMessage = '';
    let isLoading = false;
    
    async function handleLogin() {
        isLoading = true;
        errorMessage = '';
        
        try {
            // Create login payload with form information
            let login_payload = new Login(email, password);
            
            // Call Post API to try to login
            let result = await attempt_login(login_payload);

            if (result) {
                // Login successful
                console.log("Login Success");
                // Redirect to home page or dashboard
                window.location.href = '/';
            } else {
                // Login failed
                errorMessage = 'Invalid email or password';
            }
        } catch (error) {
            errorMessage = 'An error occurred during login';
            console.error('Login error:', error);
        } finally {
            isLoading = false;
        }
    }
</script>

<div class="min-h-screen flex items-center justify-center bg-[#0A1721] text-white py-12 px-4 sm:px-6">
    <div class="max-w-md p-8 space-y-8 bg-[#1E293B] rounded-lg shadow-lg mt-[-5vh] mt-80">
        <div class="text-center">
            <h1 class="text-3xl font-bold">Welcome Back</h1>
            <p class="mt-2 text-gray-400">Sign in to your account</p>
        </div>
        
        <form class="mt-8 space-y-6" on:submit|preventDefault={handleLogin}>
            {#if errorMessage}
                <div class="bg-red-900/50 border border-red-500 text-red-200 px-4 py-3 rounded relative" role="alert">
                    <span class="block sm:inline">{errorMessage}</span>
                </div>
            {/if}
            
            <div>
                <label for="email" class="block text-sm font-medium text-gray-300">Email</label>
                <input 
                    id="email" 
                    name="email" 
                    type="email" 
                    required 
                    bind:value={email}
                    class="mt-1 block w-full px-3 py-2 bg-[#0F172A] border border-gray-700 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                    placeholder="you@example.com"
                />
            </div>
            
            <div>
                <label for="password" class="block text-sm font-medium text-gray-300">Password</label>
                <input 
                    id="password" 
                    name="password" 
                    type="password" 
                    required 
                    bind:value={password}
                    class="mt-1 block w-full px-3 py-2 bg-[#0F172A] border border-gray-700 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                    placeholder="••••••••"
                />
            </div>
            
            <div class="flex items-center justify-between">
                <div class="flex items-center">
                    <input 
                        id="remember-me" 
                        name="remember-me" 
                        type="checkbox" 
                        class="h-4 w-4 bg-[#0F172A] border-gray-700 rounded"
                    />
                    <label for="remember-me" class="ml-2 block text-sm text-gray-300">Remember me</label>
                </div>
                
                <div class="text-sm">
                    <a href="/signup" class="font-medium text-indigo-400 hover:text-indigo-300">Forgot password?</a>
                </div>
            </div>
            
            <div>
                <button 
                    type="submit" 
                    class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
                    disabled={isLoading}
                >
                    {#if isLoading}
                        <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                        </svg>
                        Signing in...
                    {:else}
                        Sign in
                    {/if}
                </button>
            </div>
        </form>
        
        <div class="text-center mt-4">
            <p class="text-sm text-gray-400">
                Don't have an account? 
                <a href="/signup" class="font-medium text-indigo-400 hover:text-indigo-300">Sign up</a>
            </p>
        </div>
    </div>
</div> 