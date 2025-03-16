<script lang="ts">
	import { onMount } from 'svelte';
	import TextEditor from '$lib/components/TextEditor.svelte';
	import { loadDocument, saveDocument, Document } from '$lib/ts/document';

	// If you want to access dynamic parameters from the URL, use the `page` store
	import { page } from '$app/stores';

	$: documentId = $page.params.id; // Access the dynamic parameter from the URL
	$: console.log('Document ID:', documentId); // Reactive statement to log the documentId
	let documentData: Document | null = null; // Document Data to be parsed

	// You could use documentId to load data, or trigger actions on page load
	onMount(async () => {
		documentData = await loadDocument(Number(documentId));
	});
</script>

<main class="flex min-h-screen flex-col items-center justify-center bg-[#0A1721] text-[#E5E5E5]">
	<!-- Conditional rendering: pass documentData only when it's not null -->
	{#if documentData}
		<TextEditor {documentData} />
	{/if}
</main>
