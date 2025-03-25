<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import TextEditor from '$lib/components/TextEditor.svelte';
	import { get_document, update_document, get_document_users, setup_auto_save, type Document, type DocumentUser } from '$lib/ts/document';

	
	import { page } from '$app/stores'; // to access dynamic parameters from URL

	$: documentId = $page.params.id; // Access the dynamic parameter from the URL
	$: console.log('Document ID:', documentId); // Reactive statement to log the documentId
	let documentData: Document | null = null; // Document Data to be parsed
	let loading = true; // save state for UI
	let error = false; // save state for UI
	let lastSaveStatus: boolean | null = null; // tracks success/failure of last save operation
	let cleanupAutoSave: (() => void) | null = null; //function to stop auto-saving when page is left

	// Function to fetch and log document users
	async function fetchDocumentUsers(doc: Document) {
		console.log("Fetching document users...");
		const users = await get_document_users(doc);
		
		if (users) {
			console.log("Document users:", users);
			users.forEach(user => {
				console.log(`User: ${user.name} (${user.email}) - Role: ${user.role}`);
			});
		} else {
			console.log("No users found or error fetching users");
		}
	}

	// On page load
	onMount(async () => {
		try {
			documentData = await get_document(Number(documentId)); // get document data
			loading = false;
			
			// If we find the document data from API call
			if (documentData) {
				// test function to console log users with permissions to current doc
				fetchDocumentUsers(documentData);
				
				// Set up auto-save when document is loaded
				cleanupAutoSave = setup_auto_save(documentData, (success) => {
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

</script>

<main class="flex min-h-screen flex-col items-center justify-center bg-[#0A1721] text-[#E5E5E5]">
	<!-- Conditional rendering: pass documentData only when it's not null -->
	{#if documentData}
		<TextEditor {documentData} />
	{/if}
</main>
