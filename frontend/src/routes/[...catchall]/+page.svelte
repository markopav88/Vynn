<script lang="ts">
    import Navbar from '$lib/components/Navbar.svelte';
    import Footer from '$lib/components/Footer.svelte';
    import { page } from '$app/stores';
    
    let isLoggedIn = false;
    let path = $page.params.catchall || 'the requested page';
</script>

<div class="bg-black min-vh-100 d-flex flex-column">
    <Navbar {isLoggedIn} />
    
    <div class="container flex-grow-1 d-flex align-items-center justify-content-center py-5">
        <div class="text-center">
            <div class="mb-4">
                <i class="bi bi-tools text-green" style="font-size: 5rem;"></i>
            </div>
            <h1 class="fw-bold mb-4">Work in Progress</h1>
            <p class="fs-4 text-white-50 mb-5">
                We're currently building <span class="text-green">{path}</span>.<br>
                Check back soon for updates!
            </p>
            <a href="/" class="btn btn-green btn-lg px-4">Return to Home</a>
        </div>
    </div>
    
    <Footer />
</div> 