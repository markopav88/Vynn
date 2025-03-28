<!-- frontend/src/routes/+page.svelte -->

<!--
    This is our root main page. It's the first page that loads when we go to the root URL.
    Svelte is a framework that allows us to build web applications using HTML, CSS, and Typescript.
    In a Svelte file we can have HTML, CSS, and Typescript code.
-->
<script lang="ts">
    import { onMount } from 'svelte';
    import { logout } from "$lib/ts/user";
    import Navbar from '$lib/components/Navbar.svelte';
    import heroBackground from '$lib/assets/hero-background.png';
    
    let isLoggedIn = false;
    
    onMount(async () => {
        // Check if user is logged in
        // if so redirect to drive
    });
</script>

<div class="bg-black min-vh-100">
    <Navbar {isLoggedIn} />
    
    <!-- Hero Section with Background Image -->
    <section class="position-relative">
        <!-- Background Image -->
        <div class="position-absolute top-0 start-0 w-100 h-100" style="overflow: hidden;">
            <img src="{heroBackground}" alt="Hero Background" class="w-100 h-100 object-fit-cover opacity-25" />
        </div>
        
        <div class="container py-5">
            <!-- Header that spans full width -->
            <div class="row mb-4">
                <div class="col-12 text-center">
                    <h1 class="fw-bold" style="font-size: 6rem; line-height: 1.1;">
                        Write using the Power of<br>
                        <span class="text-green">Neovim + AI</span>
                    </h1>
                </div>
            </div>
            
            <!-- Content in two columns -->
            <div class="row py-3 align-items-center">
                <!-- Left side: Text content -->
                <div class="col-md-6 text-center" style="margin-top: -2rem;">
                    <p class="fs-4 text-white-50 mb-4">
                        Experience the perfect blend of Neovim's efficiency and AI assistance designed specifically for writers. Craft your stories, articles, and content with unprecedented speed and creativity.
                    </p>
        
                    
                    <div class="d-flex gap-4 justify-content-center">
                        <a href="/editor" class="btn btn-green btn-lg px-4 py-2">
                            Try Vynn Editor
                        </a>
                        <a href="/tutorial" class="btn btn-outline-light btn-lg px-4 py-2">
                            View Tutorial
                        </a>
                    </div>
                </div>
                
                <!-- Right side: Demo image -->
                <div class="col-md-6 mt-15 mt-md-0 d-flex justify-content-end" style="margin-top: -10rem;">
                    <div class="position-relative ms-md-5" style="margin-right: -10rem;">
                        <!-- Placeholder image -->
                        <div class="bg-dark rounded-3 shadow-lg p-3 d-flex align-items-center justify-content-center" style="width: 700px; height: 500px; border: 1px solid rgba(255,255,255,0.1);">
                            <div class="text-center">
                                <svg xmlns="http://www.w3.org/2000/svg" width="80" height="80" fill="currentColor" class="bi bi-code-square text-green mb-3" viewBox="0 0 16 16">
                                    <path d="M14 1a1 1 0 0 1 1 1v12a1 1 0 0 1-1 1H2a1 1 0 0 1-1-1V2a1 1 0 0 1 1-1zM2 0a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V2a2 2 0 0 0-2-2z"/>
                                    <path d="M6.854 4.646a.5.5 0 0 1 0 .708L4.207 8l2.647 2.646a.5.5 0 0 1-.708.708l-3-3a.5.5 0 0 1 0-.708l3-3a.5.5 0 0 1 .708 0zm2.292 0a.5.5 0 0 0 0 .708L11.793 8l-2.647 2.646a.5.5 0 0 0 .708.708l3-3a.5.5 0 0 0 0-.708l-3-3a.5.5 0 0 0-.708 0z"/>
                                </svg>
                                <p class="text-white-50">Vynn Editor Demo</p>
                                <p class="text-white-50 small">(Product screenshot will be displayed here)</p>
                            </div>
                        </div>
                        
                        <!-- Glow effect -->
                        <div class="position-absolute top-50 start-50 translate-middle" style="width: 100%; height: 100%; background: radial-gradient(circle, rgba(16,185,129,0.2) 0%, rgba(0,0,0,0) 70%); z-index: -1;"></div>
                    </div>
                </div>
            </div>
        </div>
    </section>
    
    <!-- Features Section -->
    <section class="py-5">
        <div class="container">
            <h2 class="text-center fw-bold mb-3">Powerful Features for Modern Writers</h2>
            
            <p class="text-center text-green mb-5">
                Combine the precision of Neovim with cutting-edge AI assistance to enhance your writing workflow.
            </p>
            
            <div class="row g-4">
                <!-- Feature 1 -->
                <div class="col-md-4">
                    <div class="d-flex">
                        <div class="flex-shrink-0">
                            <div class="bg-success rounded-circle d-flex align-items-center justify-content-center me-3" style="width: 40px; height: 40px;">
                                <i class="bi bi-robot text-white"></i>
                            </div>
                        </div>
                        <div>
                            <h3 class="fs-5 fw-semibold">AI-Powered Suggestions</h3>
                            <p class="text-light-emphasis small">
                                Get real-time writing suggestions, completions, and improvements powered by advanced AI models.
                            </p>
                        </div>
                    </div>
                </div>
                
                <!-- Feature 2 -->
                <div class="col-md-4">
                    <div class="d-flex">
                        <div class="flex-shrink-0">
                            <div class="bg-success rounded-circle d-flex align-items-center justify-content-center me-3" style="width: 40px; height: 40px;">
                                <i class="bi bi-keyboard text-white"></i>
                            </div>
                        </div>
                        <div>
                            <h3 class="fs-5 fw-semibold">Vim Keybindings</h3>
                            <p class="text-light-emphasis small">
                                Leverage the power of Vim keybindings for lightning-fast text editing and navigation.
                            </p>
                        </div>
                    </div>
                </div>
                
                <!-- Feature 3 -->
                <div class="col-md-4">
                    <div class="d-flex">
                        <div class="flex-shrink-0">
                            <div class="bg-success rounded-circle d-flex align-items-center justify-content-center me-3" style="width: 40px; height: 40px;">
                                <i class="bi bi-palette text-white"></i>
                            </div>
                        </div>
                        <div>
                            <h3 class="fs-5 fw-semibold">Custom Themes</h3>
                            <p class="text-light-emphasis small">
                                Personalize your writing environment with customizable themes and color schemes.
                            </p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </section>
</div>

<style>
    :global(body) {
        margin: 0;
        padding: 0;
        background-color: #000;
        color: #fff;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
    }
</style>