<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';

	import { get_document, update_document, setup_auto_save, get_project_from_document } from '$lib/ts/document';
	import { logout } from '$lib/ts/user'
	import { get_project_documents } from '$lib/ts/project';
	import { handleNormalModeKeydown } from '$lib/ts/editor-commands';

	import logo from '$lib/assets/logo.png';
	import backgroundImage from '$lib/assets/editor-background.jpg';
	import profileDefault from '$lib/assets/profile-image.png';

	import '$lib/assets/style/document.css'

	// Document state
	let documentId = $page.params.id;
	let documentData: any = null;
	let loading = true;
	let error = false;

	// Project state
	let projectDocuments: any[] = [];
	let currentDocumentIndex = -1;
	let projectDocumentsMap = new Map(); // Map to store preloaded documents

	// Editor state
	let editorContent = '';
	let editorMode = 'NORMAL';
	let cursorLine = 1; // for indicator in bottom right
	let cursorColumn = 1; // for indicator in bottom right
	let editorElement: HTMLTextAreaElement;

	// Add this for line numbers
	let lines: string[] = [''];
	let activeLineIndex = 0;

	// Add these variables for animation
	let isAnimating = false;
	let slideDirection = ''; // 'left' or 'right'
	let previousDocumentContent = '';
	let previousDocumentLines: string[] = [];
	let previousActiveLineIndex = 0;
	let animationHeight = 0; // Store the height for consistent animation

	const LINE_HEIGHT = 24; // 1.5rem = 24px (assuming 16px font size)
	const MIN_LINES = 30; // minimum lines to display
	let documentReady = false;	// to track when the document is ready to display
	let projectDocumentsLoaded = false; // to track when project documents are loaded

	// Add a variable to track when navbar should fade in
	let navbarReady = false;

	// Add this variable declaration at the script level
	let autoSaveCleanup: (() => void) | null = null;

	// Add command mode variables
	let commandInput = '';
	let commandPrefix = '';
	let commandInputElement: HTMLInputElement;
	let searchResults: number[] = [];
	let currentSearchIndex = -1;

	// Add variables for error messages
	let commandError = '';
	let commandErrorTimeout: ReturnType<typeof setTimeout> | null = null;

	// Add clipboard state
	let clipboardText = '';
	let normalModeBuffer = '';
	let normalModeBufferTimeout: ReturnType<typeof setTimeout> | null = null;

	// Add a function to prevent default browser behavior for certain key combinations
	function preventBrowserDefaults(event: KeyboardEvent) {
		// Prevent OS shortcuts by capturing all Ctrl/Cmd combinations
		if (event.ctrlKey || event.metaKey) {
			// Allow only specific browser shortcuts we want to keep
			const allowedKeys = ['c', 'v', 'a', 'z', 'y', 'f'];
			if (!allowedKeys.includes(event.key.toLowerCase())) {
				event.preventDefault();
			}
		}

		// Prevent other problematic keys
		const preventKeys = ['F1', 'F3', 'F5', 'F6', 'F7', 'F10', 'F11', 'F12'];
		if (preventKeys.includes(event.key)) {
			event.preventDefault();
		}
	}

	// Function to switch to another document with animation
	async function switchDocument(docId: number) {
		try {
			// Don't switch if already on this document
			if (docId.toString() === documentId) {
				console.log(`Already viewing document ${docId}, no switch needed`);
				return;
			}

			console.log(`Switching to document ${docId} from ${documentId}`);

			// Don't switch if already animating
			if (isAnimating) return;

			// Save current document before switching
			if (documentData) {
				console.log('Saving current document before switching');
				documentData.content = editorContent;
				await update_document(documentData);
			}

			// Check if we already have the document loaded
			if (projectDocumentsMap.has(docId)) {
				console.log('Using preloaded document data');

				// Determine slide direction based on document indices
				const currentIndex = projectDocuments.findIndex((doc) => doc.id.toString() === documentId);
				const newIndex = projectDocuments.findIndex((doc) => doc.id === docId);

				if (currentIndex < newIndex) {
					// Moving to a higher number - slide left
					slideDirection = 'left';
				} else {
					// Moving to a lower number - slide right
					slideDirection = 'right';
				}

				// Store current document content for animation
				previousDocumentContent = editorContent;
				previousDocumentLines = [...lines];
				previousActiveLineIndex = activeLineIndex;

				// Store current editor height for smooth animation
				if (editorElement && editorElement.parentElement) {
					animationHeight = Math.max(editorElement.parentElement.offsetHeight, editorElement.scrollHeight);
				}

				// Start animation
				isAnimating = true;

				// Update document ID in URL without full page reload
				window.history.pushState({}, '', `/document/${docId}`);
				documentId = docId.toString();

				// Load the preloaded document data
				documentData = projectDocumentsMap.get(docId);
				editorContent = documentData.content || '';
				lines = editorContent.split('\n');

				// Update current document index
				currentDocumentIndex = projectDocuments.findIndex((doc) => doc.id === docId);

				// Get project info for the new document to ensure project name is correct
				const projectInfo = await get_project_from_document(docId);
				if (projectInfo && projectInfo.project_id) {
					documentData.project_name = projectInfo.project_name;
				}

				// Wait for animation to complete
				setTimeout(() => {
					isAnimating = false;
					slideDirection = '';
					previousDocumentContent = '';
					previousDocumentLines = [];
					animationHeight = 0;
					// Adjust textarea height
					setTimeout(adjustTextareaHeight, 0);
				}, 300); // Match this with CSS transition duration

				return;
			}

			// If document not preloaded, navigate to it the traditional way
			console.log(`Document not preloaded, navigating to /document/${docId}`);
			window.location.href = `/document/${docId}`;
		} catch (error) {
			console.error('Error switching document:', error);
			isAnimating = false;
		}
	}

	// Update the loadProjectDocuments function to set projectDocumentsLoaded
	async function loadProjectDocuments() {
		try {
			// Get project information for this document
			const projectInfo = await get_project_from_document(parseInt(documentId));

			if (projectInfo && projectInfo.project_id) {
				// Get all documents in this project
				const documents = await get_project_documents(projectInfo.project_id);

				if (documents && documents.length > 0) {
					projectDocuments = documents;

					// Find the index of the current document
					currentDocumentIndex = projectDocuments.findIndex((doc) => doc.id.toString() === documentId);

					// Preload documents into the map
					projectDocuments.forEach((doc) => {
						projectDocumentsMap.set(doc.id, doc);
					});

					// Set the project name in documentData
					documentData.project_name = projectInfo.project_name;
				}
			}
			// Set projectDocumentsLoaded to true regardless of whether document is in a project
			projectDocumentsLoaded = true;
		} catch (error) {
			console.error('Error loading project documents:', error);
			// Still set projectDocumentsLoaded to true even if there's an error
			projectDocumentsLoaded = true;
		}
	}

	// Function to enter command mode
	function enterCommandMode(prefix: string) {
		editorMode = 'COMMAND';
		commandPrefix = prefix;
		commandInput = '';

		// Focus the command input after it renders
		setTimeout(() => {
			if (commandInputElement) {
				commandInputElement.focus();
			}
		}, 0);
	}

	// Function to exit command mode
	function exitCommandMode() {
		editorMode = 'NORMAL';
		commandInput = '';
		commandPrefix = '';
		// Return focus to editor
		if (editorElement) {
			editorElement.focus();
		}
	}

	// Function to handle command input
	function handleCommandInput() {
		if (commandPrefix === '/' || commandPrefix === '?') {
			// For search commands, update search results as user types
			performSearch();
		}
	}

	// Function to show command error for a few seconds
	function showCommandError(message: string) {
		commandError = message;

		// Clear any existing timeout
		if (commandErrorTimeout) {
			clearTimeout(commandErrorTimeout);
		}

		// Auto-hide the error after 3 seconds
		commandErrorTimeout = setTimeout(() => {
			commandError = '';
		}, 3000);
	}

	// Function to perform search based on command input
	function performSearch() {
		// Check if editor content is empty
		if (!editorContent.trim()) {
			searchResults = [];
			currentSearchIndex = -1;
			showCommandError('No content to search in');
			return;
		}

		// Check if search term is empty
		if (!commandInput.trim()) {
			searchResults = [];
			currentSearchIndex = -1;
			showCommandError('Please enter a search term');
			return;
		}

		const searchTerm = commandInput.toLowerCase();
		const content = editorContent.toLowerCase();
		const results: number[] = [];

		let index = content.indexOf(searchTerm);
		while (index !== -1) {
			results.push(index);
			index = content.indexOf(searchTerm, index + 1);
		}

		searchResults = results;

		// Show error if no results found
		if (results.length === 0) {
			showCommandError(`No matches found for "${commandInput}"`);
		} else {
			// Clear any existing error
			commandError = '';
		}

		// Set current index based on search direction
		if (searchResults.length > 0) {
			if (commandPrefix === '/') {
				// Forward search - start from the beginning
				currentSearchIndex = 0;
			} else {
				// Backward search - start from the end
				currentSearchIndex = searchResults.length - 1;
			}
		} else {
			currentSearchIndex = -1;
		}
	}

	// Function to navigate to the current search result
	function navigateToSearchResult() {
		if (searchResults.length > 0 && currentSearchIndex >= 0) {
			const position = searchResults[currentSearchIndex];

			// Set cursor position to the search result
			if (editorElement) {
				editorElement.focus();
				editorElement.setSelectionRange(position, position + commandInput.length);

				// Ensure the cursor is visible
				const textBeforeCursor = editorContent.substring(0, position);
				const lines = textBeforeCursor.split('\n');
				activeLineIndex = lines.length - 1;

				// Update cursor position
				cursorLine = lines.length;
				cursorColumn = lines[lines.length - 1].length + 1;
			}
		}
	}

	// Function to navigate through search results with n/N
	function navigateSearchResults(forward: boolean) {
		if (searchResults.length === 0) return;

		if (forward) {
			currentSearchIndex = (currentSearchIndex + 1) % searchResults.length;
		} else {
			currentSearchIndex = (currentSearchIndex - 1 + searchResults.length) % searchResults.length;
		}

		navigateToSearchResult();
	}

	// Function to handle colon commands
	function handleColonCommand(command: string) {
		// Simple command handling for now
		const cmd = command.trim().toLowerCase();

		if (cmd === 'q' || cmd === 'quit') {
			// Navigate back to drive
			goto('/drive');
		} else if (cmd === 'w' || cmd === 'write') {
			// Save the document
			if (documentData) {
				documentData.content = editorContent;
				update_document(documentData);
			}
		} else if (cmd === 'wq') {
			// Save and quit
			if (documentData) {
				documentData.content = editorContent;
				update_document(documentData).then(() => {
					goto('/drive');
				});
			}
		} else {
			// Show error for unrecognized command
			showCommandError(`Unknown command: "${command}"`);
			return false;
		}

		return true;
	}

	// Function to handle normal mode key sequences
	function handleNormalModeSequence(key: string) {
		// Add the key to the buffer
		normalModeBuffer += key;

		// Clear any existing timeout
		if (normalModeBufferTimeout) {
			clearTimeout(normalModeBufferTimeout);
		}

		// Set a timeout to clear the buffer after a delay
		normalModeBufferTimeout = setTimeout(() => {
			normalModeBuffer = '';
		}, 800); // 800ms timeout for multi-key commands

		// Check for sequences
		if (normalModeBuffer === 'yy') {
			// Copy the current line or selection
			copyText();
			normalModeBuffer = ''; // Clear buffer after command
			return true;
		} else if (normalModeBuffer === 'dd') {
			// Delete the current line
			deleteCurrentLine();
			normalModeBuffer = ''; // Clear buffer after command
			return true;
		}

		return false;
	}

	// Function to copy text
	function copyText() {
		if (!editorElement) return;

		// Get the selection or current line
		let textToCopy = '';

		if (editorElement.selectionStart !== editorElement.selectionEnd) {
			// Copy selected text
			textToCopy = editorContent.substring(editorElement.selectionStart, editorElement.selectionEnd);
		} else {
			// Copy current line if no selection
			const lines = editorContent.split('\n');
			textToCopy = lines[activeLineIndex];
		}

		// Store in our internal clipboard
		clipboardText = textToCopy;

		// Also copy to system clipboard if possible
		try {
			navigator.clipboard.writeText(textToCopy).then(() => {
				showCommandError('Text copied to clipboard');
			});
		} catch (e) {
			// Fallback for browsers that don't support clipboard API
			showCommandError('Text copied');
		}
	}

	// Function to delete text - update to adjust height after deletion
	function deleteText() {
		if (!editorElement) return;

		// Check if there's a selection
		if (editorElement.selectionStart !== editorElement.selectionEnd) {
			// Get the selection range
			const start = editorElement.selectionStart;
			const end = editorElement.selectionEnd;

			// Delete the selected text
			editorContent = editorContent.substring(0, start) + editorContent.substring(end);

			// Update the editor
			editorElement.value = editorContent;
			editorElement.setSelectionRange(start, start);

			// Update lines array for line numbers
			lines = editorContent.split('\n');

			// Update cursor position
			updateCursorPosition();

			// Adjust textarea height to shrink if needed
			adjustTextareaHeight();
		}
	}

	// Function to paste text
	function pasteText() {
		if (!editorElement || !clipboardText) return;

		// Get the cursor position
		const start = editorElement.selectionStart;
		const end = editorElement.selectionEnd;

		// Insert the clipboard text
		editorContent = editorContent.substring(0, start) + clipboardText + editorContent.substring(end);

		// Update the editor
		editorElement.value = editorContent;
		editorElement.setSelectionRange(start + clipboardText.length, start + clipboardText.length);

		// Update lines array for line numbers
		lines = editorContent.split('\n');

		// Update cursor position
		updateCursorPosition();

		// Adjust textarea height to accommodate new content
		adjustTextareaHeight();

		// Ensure the cursor is visible by scrolling if needed
		setTimeout(() => {
			// Calculate which line the cursor is on
			const textBeforeCursor = editorContent.substring(0, start + clipboardText.length);
			const linesBeforeCursor = textBeforeCursor.split('\n');
			const cursorLineIndex = linesBeforeCursor.length - 1;

			// Scroll to make the cursor visible
			const lineHeight = LINE_HEIGHT; // Using your defined line height constant
			const scrollTop = cursorLineIndex * lineHeight;

			if (editorElement) {
				editorElement.scrollTop = scrollTop;
			}
		}, 0);
	}

	// Function to delete the current line - update to adjust height after deletion
	function deleteCurrentLine() {
		if (!editorElement) return;

		// Get the lines
		const lines = editorContent.split('\n');

		// Make sure we have a valid line index
		if (activeLineIndex >= 0 && activeLineIndex < lines.length) {
			// Remove the current line
			lines.splice(activeLineIndex, 1);

			// If we removed the last line and there are no lines left, add an empty line
			if (lines.length === 0) {
				lines.push('');
			}

			// If we removed the last line, move cursor up
			if (activeLineIndex >= lines.length) {
				activeLineIndex = Math.max(0, lines.length - 1);
			}

			// Update editor content
			editorContent = lines.join('\n');

			// Update the editor
			editorElement.value = editorContent;

			// Position cursor at the beginning of the line
			const newPosition = getPositionFromLineIndex(activeLineIndex);
			editorElement.setSelectionRange(newPosition, newPosition);

			// Update lines array for line numbers
			updateLines();

			// Update cursor position
			updateCursorPosition();

			// Adjust textarea height to shrink if needed
			adjustTextareaHeight();

			// Show feedback
			showCommandError('Line deleted');
		}
	}

	// Helper function to update lines array
	function updateLines() {
		lines = editorContent.split('\n');
	}

	// Helper function to get text position from line index
	function getPositionFromLineIndex(lineIndex: number): number {
		const lines = editorContent.split('\n');
		let position = 0;

		for (let i = 0; i < lineIndex; i++) {
			position += lines[i].length + 1; // +1 for the newline character
		}

		return position;
	}

	// Update handleKeyDown to support the new commands
	function handleKeyDown(event: KeyboardEvent) {
		// First prevent any OS bindings
		preventBrowserDefaults(event);

		// Handle document switching with Ctrl+number in any mode
		if (event.ctrlKey && !event.altKey && !event.metaKey && !event.shiftKey) {
			// Check if the key is a number from 1-9
			const numKey = parseInt(event.key);
			if (!isNaN(numKey) && numKey >= 1 && numKey <= 9) {
				// Check if we have a document at this index
				if (projectDocuments.length >= numKey) {
					// Get the document ID at index (numKey-1)
					const docId = projectDocuments[numKey - 1].id;

					// Switch to that document
					switchDocument(docId);
					event.preventDefault();
					return;
				}
			}
		}

		// In NORMAL mode, prevent most key inputs
		if (editorMode === 'NORMAL') {
			// Always prevent default for most keys in NORMAL mode
			const allowedKeys = [
				'Escape',
				'ArrowUp',
				'ArrowDown',
				'ArrowLeft',
				'ArrowRight',
				'Home',
				'End',
				'PageUp',
				'PageDown',
				'Tab'
			];

			// Also allow command keys
			const commandKeys = [':', '/', '?', 'i', 'x', 'y', 'p', 'd'];

			// Allow Ctrl combinations
			if (!event.ctrlKey && !allowedKeys.includes(event.key) && !commandKeys.includes(event.key)) {
				event.preventDefault();
			}

			// Handle mode switches
			if (event.key === 'i') {
				editorMode = 'INSERT';
				event.preventDefault();
				return;
			} else if (event.key === ':') {
				enterCommandMode(':');
				event.preventDefault();
				return;
			} else if (event.key === '/') {
				enterCommandMode('/');
				event.preventDefault();
				return;
			} else if (event.key === '?') {
				enterCommandMode('?');
				event.preventDefault();
				return;
			} else if (event.key === 'x') {
				// Delete selected text
				deleteText();
				event.preventDefault();
				return;
			} else if (event.key === 'p') {
				// Paste text
				pasteText();
				event.preventDefault();
				return;
			} else if (event.key === 'd') {
				// Check for 'dd' sequence
				const handled = handleNormalModeSequence('d');
				if (handled) {
					event.preventDefault();
					return;
				}
			} else if (event.key === 'y') {
				// Check for 'yy' sequence
				const handled = handleNormalModeSequence('y');
				if (handled) {
					event.preventDefault();
					return;
				}
			}

			// Use our normal mode handler for navigation
			handleNormalModeKeydown(event, editorElement);
		} else if (editorMode === 'INSERT') {
			// In INSERT mode, we don't need to prevent most keys
			// Just handle Escape to exit INSERT mode
		}

		if (event.key === 'Escape') {
			editorMode = 'NORMAL';
			event.preventDefault();
		}

		// Always update cursor position
		updateCursorPosition();

		// Add this at the end of the function
		updateLineNumbers();
	}

	// Update the updateCursorPosition function to be more accurate
	function updateCursorPosition() {
		if (editorElement) {
			const position = editorElement.selectionStart;
			const text = editorElement.value;
			const textBeforeCursor = text.substring(0, position);
			const lines = textBeforeCursor.split('\n');

			cursorLine = lines.length;
			cursorColumn = lines[lines.length - 1].length + 1;
			activeLineIndex = lines.length - 1;
		}
	}

	// Update the adjustTextareaHeight function to handle both growing and shrinking
	function adjustTextareaHeight() {
		if (!editorElement) return;

		// Reset height to auto to get the correct scrollHeight
		editorElement.style.height = 'auto';

		// Calculate number of lines in the content
		const numberOfLines = editorContent.split('\n').length;
		
		// Calculate the height based on number of lines
		const contentHeight = numberOfLines * LINE_HEIGHT;
		
		// Calculate the minimum height based on MIN_LINES
		const minHeight = LINE_HEIGHT * MIN_LINES;

		// Set height to the larger of content height or minimum height
		const newHeight = Math.max(contentHeight, minHeight);

		// Apply the new height to textarea with overflow handling
		editorElement.style.height = `${newHeight}px`;
		editorElement.style.overflowY = contentHeight > minHeight ? 'auto' : 'hidden';

		// Update the line numbers container height and overflow
		const lineNumbersContainer = document.querySelector('.line-numbers') as HTMLElement;
		if (lineNumbersContainer) {
			lineNumbersContainer.style.height = `${newHeight}px`;
			lineNumbersContainer.style.overflowY = contentHeight > minHeight ? 'hidden' : 'hidden';
		}

		// Update the editor container height and overflow
		const editorContainer = document.querySelector('.editor-container') as HTMLElement;
		if (editorContainer) {
			editorContainer.style.height = `${newHeight}px`;
			editorContainer.style.minHeight = `${newHeight}px`;
			editorContainer.style.overflowY = contentHeight > minHeight ? 'auto' : 'hidden';
		}

		// Update the editor wrapper to handle overflow
		const editorWrapper = document.querySelector('.editor-wrapper') as HTMLElement;
		if (editorWrapper) {
			editorWrapper.style.height = `${newHeight}px`;
			editorWrapper.style.overflowY = contentHeight > minHeight ? 'auto' : 'hidden';
		}
	}

	// Update the onMount function to set documentReady and navbarReady
	onMount(() => {
		const loadData = async () => {
			try {
				// Load the document
				documentData = await get_document(parseInt(documentId));

				if (documentData) {
					editorContent = documentData.content || '';
					lines = editorContent.split('\n');

					// Check if document is part of a project
					if (!documentData.project_id) {
						// If project_id is not in document data, try to get it from the API
						const projectData = await get_project_from_document(parseInt(documentId));
						if (projectData && projectData.project_id) {
							// Add project_id to document data
							documentData.project_id = projectData.project_id;
						}
					}

					// Now autoSaveCleanup is defined when this assignment happens
					autoSaveCleanup = setup_auto_save(documentData, () => {
						if (documentData) {
							documentData.content = editorContent;
							update_document(documentData);
						}
					});

					// Load project documents if this document is part of a project
					await loadProjectDocuments();

					// Set documentReady to true after everything is loaded
					documentReady = true;

					// Set navbarReady after a delay to create staggered animation
					setTimeout(() => {
						navbarReady = true;
					}, 400); // Delay navbar animation to happen after document picker

					// Add this line to update line numbers when document loads
					updateLineNumbers();
				} else {
					error = true;
				}
			} catch (e) {
				console.error('Error loading document:', e);
				error = true;
			} finally {
				loading = false;
			}
		};

		loadData();

		return () => {
			if (autoSaveCleanup) {
				autoSaveCleanup();
			}
		};
	});

	// Update the handleInput function to properly handle pasted content
	function handleInput(event: Event) {
		// If in NORMAL mode, prevent typing by reverting the content
		if (editorMode === 'NORMAL') {
			// Get the current selection
			const selectionStart = editorElement.selectionStart;
			const selectionEnd = editorElement.selectionEnd;

			// Revert to previous content
			editorElement.value = editorContent;

			// Restore selection
			editorElement.setSelectionRange(selectionStart - 1, selectionEnd - 1);

			// Prevent the input
			event.preventDefault();
		} else {
			// In INSERT mode, update the content and line numbers
			editorContent = editorElement.value;
			updateLineNumbers();
			adjustTextareaHeight();
		}
	}

	// Improved updateLineNumbers function
	function updateLineNumbers() {
		// Split content by newlines to get lines
		lines = editorContent.split('\n');
		
		// Ensure we have at least one line
		if (lines.length === 0) {
			lines = [''];
		}
		
		// Update active line index based on cursor position
		if (editorElement) {
			const cursorPos = editorElement.selectionStart;
			const textBeforeCursor = editorContent.substring(0, cursorPos);
			activeLineIndex = (textBeforeCursor.match(/\n/g) || []).length;
			
			// Update cursor line and column for status bar
			cursorLine = activeLineIndex + 1;
			const lastNewlinePos = textBeforeCursor.lastIndexOf('\n');
			cursorColumn = lastNewlinePos === -1 ? cursorPos + 1 : cursorPos - lastNewlinePos;
		}
	}

	// Handle cleanup in onDestroy instead
	onDestroy(() => {
		if (documentData) {
			if (autoSaveCleanup) autoSaveCleanup();
		}
	});

	// Update the search function to search from cursor position
	function searchDocument(searchTerm: string, backwards = false) {
		if (!searchTerm) return;
		
		// Clear previous search results
		searchResults = [];
		currentSearchIndex = -1;
		
		// Get current cursor position in the document
		const cursorPosition = getCursorPosition();
		
		// Find all occurrences of the search term
		let index = -1;
		const term = searchTerm.toLowerCase();
		const content = editorContent.toLowerCase();
		
		while ((index = content.indexOf(term, index + 1)) !== -1) {
			searchResults.push(index);
		}
		
		if (searchResults.length === 0) {
			showCommandError(`No matches found for '${searchTerm}'`);
			return;
		}
		
		// Find the next/previous match relative to cursor position
		if (backwards) {
			// Find the closest result that comes before the cursor
			currentSearchIndex = searchResults.findIndex(pos => pos >= cursorPosition) - 1;
			if (currentSearchIndex < 0) currentSearchIndex = searchResults.length - 1; // Wrap around
		} else {
			// Find the closest result that comes after the cursor
			currentSearchIndex = searchResults.findIndex(pos => pos >= cursorPosition);
			if (currentSearchIndex === -1) currentSearchIndex = 0; // Wrap around
		}
		
		// Navigate to the found position
		navigateToSearchResult();
	}

	// Update the executeCommand function to handle both / and ? commands
	function executeCommand(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			event.preventDefault();
			
			if (commandPrefix === '/') {
				// Forward search from cursor
				searchDocument(commandInput, false);
			} else if (commandPrefix === '?') {
				// Backward search from cursor
				searchDocument(commandInput, true);
			}
			// Handle other commands...
			
			// Exit command mode
			setEditorMode('NORMAL');
		}
	}

	// Add this function to get the current cursor position
	function getCursorPosition(): number {
		if (!editorElement) return 0;
		return editorElement.selectionStart;
	}

	// Add this function to set the editor mode
	function setEditorMode(mode: string) {
		editorMode = mode;
		
		// If switching to NORMAL mode, ensure editor has focus
		if (mode === 'NORMAL' && editorElement) {
			editorElement.focus();
		}
	}

	// Add this to your script section
	function handlePaste(event: ClipboardEvent) {
		// Let the paste happen normally
		setTimeout(() => {
			// After paste completes, update the line numbers
			editorContent = editorElement.value;
			updateLineNumbers();
			adjustTextareaHeight();
		}, 0);
	}

	// Add handler for account page navigation
    function goToAccount() {
        window.location.href = '/account';
    }

	async function handleLogout() {
        try {
            console.log("Attempting to logout...");
            const success = await logout();
            
            if (success) {
                console.log("Logout successful, redirecting to homepage");
                window.location.href = '/';
            } else {
                console.error("Logout failed");
                alert("Failed to logout. Please try again.");
            }
        } catch (error) {
            console.error("Error during logout:", error);
        }
    }
</script>

<svelte:head>
	<title>{documentData ? documentData.name : 'Document'} | Vynn</title>
</svelte:head>

<div class="editor-page">
	<div class="background-image" style="background-image: url({backgroundImage})"></div>

	<!-- Minimal Navbar with fade-in animation -->
	<div class="navbar-container" class:fade-in={navbarReady}>
		<nav class="navbar">
			<a href="/drive" class="logo-link" aria-label="Go to Drive">
				<div class="logo-container">
					<img src={logo} alt="Vynn Logo" class="logo" />
					<span class="logo-text">Vynn</span>
				</div>
			</a>
			<div class="spacer"></div>
			<div class="dropdown">
				<button 
					class="btn p-0 border-0 bg-transparent" 
					data-bs-toggle="dropdown"
					aria-expanded="false"
					aria-haspopup="true"
					aria-label="Profile menu"
				>
					<img 
						src={profileDefault} 
						alt="Profile" 
						class="rounded-circle profile-img"
						style="width: 40px; height: 40px; border: 2px solid var(--color-primary); margin-right: 10px;"
					/>
				</button>
				<ul class="dropdown-menu dropdown-menu-end dropdown-menu-dark">
					<li>
						<button class="dropdown-item" on:click={goToAccount}>
							<i class="bi bi-person me-2"></i> My Account
						</button>
					</li>
					<li><hr class="dropdown-divider"></li>
					<li>
						<button class="dropdown-item text-danger" on:click={handleLogout}>
							<i class="bi bi-box-arrow-right me-2"></i> Sign Out
						</button>
					</li>
				</ul>
			</div>
		</nav>
	</div>

	<!-- Project Document Switcher -->
	{#if projectDocumentsLoaded}
		<div class="document-switcher fade-in">
			{#if projectDocuments.length > 0}
				{#each projectDocuments as doc, index}
					<button
						class="doc-button"
						class:active={doc.id.toString() === documentId}
						on:click={() => switchDocument(doc.id)}
						disabled={doc.id.toString() === documentId}
					>
						{index + 1}
					</button>
				{/each}
			{:else}
				<button
					class="doc-button active"
					disabled
				>
					1
				</button>
			{/if}
		</div>
	{/if}

	<!-- Editor Container with animation -->
	<div class="editor-container" class:fade-in={documentReady}>
		{#if loading}
			<div class="loading">Loading document...</div>
		{:else if error}
			<div class="error">Error loading document</div>
		{:else}
			<!-- Previous document (for animation) -->
			{#if isAnimating && previousDocumentContent}
				<div
					class="editor-wrapper previous {slideDirection}-exit"
					style={animationHeight ? `height: ${animationHeight}px` : ''}
				>
					<div class="editor-content">
						<div class="line-numbers">
							{#each previousDocumentLines as line, i}
								<div class="line-number {i === previousActiveLineIndex ? 'active' : ''}">{i + 1}</div>
							{/each}
						</div>
						<div class="editor-textarea-static">{previousDocumentContent}</div>
					</div>
				</div>
			{/if}

			<!-- Current document -->
			<div
				class="editor-wrapper current {isAnimating ? `${slideDirection}-enter` : ''}"
				style={animationHeight ? `height: ${animationHeight}px` : ''}
			>
				<div class="editor-content">
					<div class="line-numbers">
						{#each Array(lines.length) as _, i}
							<div class="line-number {i === activeLineIndex ? 'active' : ''}">{i + 1}</div>
						{/each}
					</div>
					<textarea
						bind:this={editorElement}
						bind:value={editorContent}
						on:keydown={handleKeyDown}
						on:input={handleInput}
						on:paste={handlePaste}
						class="editor-textarea"
						spellcheck="false"
						autocomplete="off"
						autocapitalize="off"
						{...{ autocorrect: 'off' } as any}
					></textarea>
				</div>
			</div>
		{/if}
	</div>

	<!-- Fixed Status Bar - moved outside the editor wrapper -->
	<div class="status-bar">
		<div class="mode-indicator">
			<span class="mode {editorMode.toLowerCase()}">{editorMode}</span>
			{#if editorMode === 'COMMAND'}
				<div class="command-container">
					<span class="command-prefix">{commandPrefix}</span>
					<input
						bind:this={commandInputElement}
						bind:value={commandInput}
						on:input={handleCommandInput}
						on:keydown={executeCommand}
						class="command-input"
						autocomplete="off"
						spellcheck="false"
					/>
					{#if commandError}
						<div class="command-error">{commandError}</div>
					{/if}
				</div>
			{/if}
		</div>

		<div class="document-name">
			<span>
				{#if documentData?.project_name}
					{documentData.project_name}/{documentData.name || 'Untitled'}
				{:else}
					{documentData?.name || 'Untitled'}
				{/if}
			</span>
		</div>

		<div class="cursor-position">
			<span>Line: {cursorLine}, Col: {cursorColumn}</span>
		</div>
	</div>
</div>
