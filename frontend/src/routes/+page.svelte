// frontend/src/routes/+page.svelte

/*

    This is our main page. It's the first page that loads when we go to the root URL.
    Svelte is a framework that allows us to build web applications using HTML, CSS, and Typescript.
    In a Svelte file we can have HTML, CSS, and Typescript code.
*/
<script lang="ts">
    import { onMount } from 'svelte';

    let message = '';
    let error = '';
    let loading = false;

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
</script>

<main>
    <h1>Fullstack App</h1>
    
    <button on:click={fetchMessage} disabled={loading}>
        {loading ? 'Loading...' : 'Get Message from Backend'}
    </button>

    {#if message}
        <p class="message">{message}</p>
    {/if}

    {#if error}
        <p class="error">Error: {error}</p>
    {/if}
</main>

<style>
    main {
        padding: 2rem;
        max-width: 800px;
        margin: 0 auto;
    }

    .message {
        color: green;
        padding: 1rem;
        border: 1px solid green;
        border-radius: 4px;
    }

    .error {
        color: red;
        padding: 1rem;
        border: 1px solid red;
        border-radius: 4px;
    }

    button {
        padding: 0.5rem 1rem;
        margin: 1rem 0;
    }
</style>