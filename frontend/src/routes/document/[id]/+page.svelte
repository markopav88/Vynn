<script lang="ts">
    import TextEditor from '$lib/components/TextEditor.svelte';
    /*
    import { page } from '$app/stores';
    
    // Access the dynamic parameter from the URL
    $: documentId = $page.params.id;
    
    // You could fetch document data here based on the ID
    $: console.log('Document ID:', documentId);
    */
</script>

<main class="min-h-screen bg-[#0A1721] flex flex-col items-center justify-center text-[#E5E5E5]">
    <TextEditor />
</main> 