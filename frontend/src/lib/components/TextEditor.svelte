<script lang="ts">
	import { onMount, afterUpdate } from 'svelte';
	export let documentData: {
		id: number;
		name: string;
		content: string;
		created_at: string;
		updated_at: string;
	};

	// Editor state
	let editorContent = documentData.content || '';
	let undoStack: string[] = [];
	let redoStack: string[] = [];
	let isBold = false;
	let isItalic = false;
	let isUnderline = false;
	let textArea: HTMLTextAreaElement;
	let lastSavedContent = '';
	let saveTimeout: ReturnType<typeof setTimeout> | null = null;

	// Save current state for undo with debouncing
	function saveState() {
		if (saveTimeout) {
			clearTimeout(saveTimeout);
		}

		saveTimeout = setTimeout(() => {
			if (editorContent !== lastSavedContent) {
				undoStack.push(lastSavedContent);
				undoStack = undoStack; // Trigger reactivity
				redoStack = []; // Clear redo stack on new changes
				lastSavedContent = editorContent;
				
				// Update the document content
				documentData.content = editorContent;
			}
		}, 500);
	}

	// Handle undo
	function undo() {
		if (saveTimeout) {
			clearTimeout(saveTimeout);
			saveTimeout = null;
			if (editorContent !== lastSavedContent) {
				undoStack.push(lastSavedContent);
				lastSavedContent = editorContent;
			}
		}

		if (undoStack.length > 0) {
			redoStack.push(editorContent);
			editorContent = undoStack.pop() || '';
			lastSavedContent = editorContent;
			undoStack = undoStack;
			redoStack = redoStack;
		}
	}

	// Handle redo
	function redo() {
		if (saveTimeout) {
			clearTimeout(saveTimeout);
			saveTimeout = null;
		}

		if (redoStack.length > 0) {
			undoStack.push(editorContent);
			editorContent = redoStack.pop() || '';
			lastSavedContent = editorContent;
			undoStack = undoStack;
			redoStack = redoStack;
		}
	}

	// Handle keyboard shortcuts
	function handleKeydown(event: KeyboardEvent) {
		if (event.ctrlKey || event.metaKey) {
			switch (event.key.toLowerCase()) {
				case 'z':
					if (event.shiftKey) {
						event.preventDefault();
						redo();
					} else {
						event.preventDefault();
						undo();
					}
					break;
				case 'b':
					event.preventDefault();
					isBold = !isBold;
					break;
				case 'i':
					event.preventDefault();
					isItalic = !isItalic;
					break;
				case 'u':
					event.preventDefault();
					isUnderline = !isUnderline;
					break;
			}
		}
	}

	// Handle content changes
	function handleInput() {
		saveState();
	}

	// Keep documentData in sync with editorContent
	afterUpdate(() => {
		documentData.content = editorContent;
	});

	onMount(() => {
		if (textArea) {
			textArea.focus();
		}
	});
</script>

<div class="flex h-screen w-full">
	<textarea
		class="editor-textarea"
		bind:this={textArea}
		bind:value={editorContent}
		on:input={handleInput}
		on:keydown={handleKeydown}
		placeholder="Start writing..."
		style="font-weight: {isBold ? 'bold' : 'normal'};
               font-style: {isItalic ? 'italic' : 'normal'};
               text-decoration: {isUnderline ? 'underline' : 'none'};"
	></textarea>
</div>
