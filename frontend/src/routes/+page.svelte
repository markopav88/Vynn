<!-- frontend/src/routes/+page.svelte -->

<!--
    This is our root main page. It's the first page that loads when we go to the root URL.
    Svelte is a framework that allows us to build web applications using HTML, CSS, and Typescript.
    In a Svelte file we can have HTML, CSS, and Typescript code.
-->
<script lang="ts">
    import { logout } from "$lib/ts/login";
    
    async function handleLogout() {
        try {
            await logout();
            // Refresh the page after logout
            window.location.reload();
        } catch (error) {
            console.error("Logout error:", error);
            alert("Failed to logout. Please try again.");
        }
    }
</script>

<main class="min-h-screen bg-[#0A1721] flex flex-col items-center justify-center text-[#E5E5E5]">
    <h1 class="text-4xl mb-4">Home</h1>
    <p class="text-lg max-w-2xl text-center mb-8">
        This is the Home page. You can add your content here.
    </p>
    
    <button 
        on:click={handleLogout}
        class="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-md transition-colors"
    >
        Test Logout
    </button>
</main> 