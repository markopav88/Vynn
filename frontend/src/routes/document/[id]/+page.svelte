<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import TextEditor from '$lib/components/TextEditor.svelte';
	import { loadDocument, updateDocument, setupAutoSave, type Document } from '$lib/ts/document';

	
	import { page } from '$app/stores'; // to access dynamic parameters from URL

	$: documentId = $page.params.id; // Access the dynamic parameter from the URL
	$: console.log('Document ID:', documentId); // Reactive statement to log the documentId
	let documentData: Document | null = null; // Document Data to be parsed
	let loading = true;
	let error = false;
	let lastSaveStatus: boolean | null = null;
	let cleanupAutoSave: (() => void) | null = null;

	// On page load
	onMount(async () => {
		try {
			documentData = await loadDocument(Number(documentId)); // get document data
			loading = false;
			
			// If we find the document data from API call
			if (documentData) {
				// Set up auto-save when document is loaded
				cleanupAutoSave = setupAutoSave(documentData, (success) => {
					lastSaveStatus = success;
					// You could update UI to show save status
				});
			} else {
				error = true;
			}
		} catch (e) {
			console.error('Error loading document:', e);
			loading = false;
			error = true;
		}
	});

	onDestroy(() => {
		// Clean up auto-save when component is destroyed
		if (cleanupAutoSave) {
			cleanupAutoSave();
		}
	});

	// Manual save function for when we want to bind this 
	async function saveDocument() {
		if (documentData) {
			lastSaveStatus = await updateDocument(documentData);
		}
	}
</script>

<main class="flex min-h-screen flex-col items-center justify-center bg-[#0A1721] text-[#E5E5E5]">
	<!-- Conditional rendering: pass documentData only when it's not null -->
	{#if documentData}
		<TextEditor {documentData} />
	{/if}
</main>
