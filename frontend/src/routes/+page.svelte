<!-- frontend/src/routes/+page.svelte -->

<!--
    This is our root main page. It's the first page that loads when we go to the root URL.
    Svelte is a framework that allows us to build web applications using HTML, CSS, and Typescript.
    In a Svelte file we can have HTML, CSS, and Typescript code.
-->
<script lang="ts">
    import { onMount } from 'svelte';
    import TextEditor from '$lib/components/TextEditor.svelte';

    let message = '';
    let error = '';
    let loading = false;
    
    /* Example backend interaction
    async function fetchMessage() {
        loading = true;
        error = '';
        
        try {
            const response = await fetch('http://localhost:3000/api/hello');
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            const data = await response.json();
            message = data.message;
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to fetch message';
            console.error('Error:', e);
        } finally {
            loading = false;
        }
    }
    

    // Optional: fetch on page load
    onMount(() => {
        fetchMessage();
    });
    */
</script>

<main class="min-h-screen bg-[#0A1721] flex flex-col items-center justify-center text-[#E5E5E5]">
    <h1 class="text-4xl mb-4">Home</h1>
    <p class="text-lg max-w-2xl text-center">
        This is the Home page. You can add your content here.
    </p>
</main> 