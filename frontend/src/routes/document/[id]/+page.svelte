<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { jsPDF } from 'jspdf';

	import { get_document, update_document, setup_auto_save, get_project_from_document } from '$lib/ts/document';
	import { logout, get_current_user, get_profile_image_url } from '$lib/ts/user'
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

	// Constants for editor configuration
	const LINE_HEIGHT = 24; // 1.5rem = 24px (assuming 16px font size)
	const MIN_LINES = 30; // minimum lines to display
	const MAX_COLUMN_WIDTH = 111; // maximum characters per line
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

	// Add state for commands overlay
	let showCommands = false;

	// User profile data
	let userId: number | null = null;
	let userProfileImage = profileDefault;

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

	// Update handleKeyDown to handle arrow keys consistently in both modes
	function handleKeyDown(event: KeyboardEvent) {
		// First prevent any OS bindings
		preventBrowserDefaults(event);

		// Handle document switching with Ctrl+number in any mode
		if (event.ctrlKey && !event.altKey && !event.metaKey && !event.shiftKey) {
			const numKey = parseInt(event.key);
			if (!isNaN(numKey) && numKey >= 1 && numKey <= 9) {
				if (projectDocuments.length >= numKey) {
					const docId = projectDocuments[numKey - 1].id;
					switchDocument(docId);
					event.preventDefault();
					return;
				}
			}
		}

		// Special handling for arrow keys in both modes
		if (event.key === 'ArrowUp' || event.key === 'ArrowDown') {
			const totalLines = editorContent.split('\n').length;
			const currentPosition = editorElement.selectionStart;
			const textBeforeCursor = editorContent.substring(0, currentPosition);
			const currentLineNumber = textBeforeCursor.split('\n').length;

			// Handle bounds for both modes
			if (event.key === 'ArrowDown' && currentLineNumber >= totalLines) {
				event.preventDefault();
				cursorLine = totalLines;
				activeLineIndex = totalLines - 1;
				updateLineNumbers();
				return;
			} else if (event.key === 'ArrowUp' && currentLineNumber <= 1) {
				event.preventDefault();
				cursorLine = 1;
				activeLineIndex = 0;
				updateLineNumbers();
				return;
			}

			// Let the arrow key event happen
			setTimeout(() => {
				updateCursorPosition();
				updateLineNumbers();
			}, 0);
		}

		// Normal mode specific handling
		if (editorMode === 'NORMAL') {
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

			const commandKeys = [':', '/', '?', 'i', 'x', 'y', 'p', 'd'];

			if (!event.ctrlKey && !allowedKeys.includes(event.key) && !commandKeys.includes(event.key)) {
				event.preventDefault();
			}

			// Handle mode switches and commands
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
				deleteText();
				event.preventDefault();
				return;
			} else if (event.key === 'p') {
				pasteText();
				event.preventDefault();
				return;
			} else if (event.key === 'd' || event.key === 'y') {
				const handled = handleNormalModeSequence(event.key);
				if (handled) {
					event.preventDefault();
					return;
				}
			}

			handleNormalModeKeydown(event, editorElement);
		}

		if (event.key === 'Escape') {
			editorMode = 'NORMAL';
			event.preventDefault();
		}

		// Update cursor position and line numbers for non-arrow key events
		if (event.key !== 'ArrowUp' && event.key !== 'ArrowDown') {
			updateCursorPosition();
			updateLineNumbers();
		}
	}

	// Update the updateCursorPosition function to ensure accurate line highlighting
	function updateCursorPosition() {
		if (!editorElement) return;
		
		const position = editorElement.selectionStart;
		const text = editorContent;
		const totalLines = text.split('\n').length;
		
		// Find all newline positions before cursor
		const textBeforeCursor = text.substring(0, position);
		const newlines = [...textBeforeCursor.matchAll(/\n/g)];
		
		// Calculate current line (1-based) and active line index (0-based)
		cursorLine = newlines.length + 1;
		activeLineIndex = newlines.length; // This should match the cursor line minus 1
		
		// Calculate column position
		const lastNewlinePos = textBeforeCursor.lastIndexOf('\n');
		cursorColumn = lastNewlinePos === -1 ? position + 1 : position - lastNewlinePos;
		
		// Handle bounds checking
		if (cursorLine > totalLines) {
			cursorLine = totalLines;
			activeLineIndex = totalLines - 1;
		} else if (cursorLine < 1) {
			cursorLine = 1;
			activeLineIndex = 0;
		}
		
		// Double-check bounds for safety
		activeLineIndex = Math.min(Math.max(0, activeLineIndex), totalLines - 1);
	}

	// Update the adjustTextareaHeight function to handle full page scrolling
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

		// Set height to the larger of content height or minimum height, plus extra padding
		const newHeight = Math.max(contentHeight, minHeight) + 48; // Add extra padding

		// Apply the new height to textarea without overflow
		editorElement.style.height = `${newHeight}px`;
		editorElement.style.overflowY = 'hidden';

		// Update the line numbers container height
		const lineNumbersContainer = document.querySelector('.line-numbers') as HTMLElement;
		if (lineNumbersContainer) {
			lineNumbersContainer.style.height = `${newHeight}px`;
			lineNumbersContainer.style.overflowY = 'hidden';
		}

		// Update the editor container height
		const editorContainer = document.querySelector('.editor-container') as HTMLElement;
		if (editorContainer) {
			editorContainer.style.height = `${newHeight}px`;
			editorContainer.style.minHeight = `${newHeight}px`;
			editorContainer.style.overflowY = 'hidden';
		}

		// Update the editor wrapper
		const editorWrapper = document.querySelector('.editor-wrapper') as HTMLElement;
		if (editorWrapper) {
			editorWrapper.style.height = `${newHeight}px`;
			editorWrapper.style.overflowY = 'hidden';
		}

		// Update the editor page container to be at least as tall as the editor
		const editorPage = document.querySelector('.editor-page') as HTMLElement;
		if (editorPage) {
			editorPage.style.minHeight = `${newHeight + 200}px`; // Add extra space for navbar and status bar
		}
	}

	// Update the onMount function to set documentReady and navbarReady
	onMount(async () => {
		// Load document data and profile image in parallel
		try {
			const [docResult, userResult] = await Promise.all([
				loadDocumentData(),
				loadUserProfile()
			]);
		} catch (e) {
			console.error('Error during initialization:', e);
			error = true;
		} finally {
			loading = false;
		}
	});

	// Function to load document data
	async function loadDocumentData() {
		try {
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

				// Load project documents if this document is part of a project
				await loadProjectDocuments();

				// Set up auto-save
				autoSaveCleanup = setup_auto_save(documentData, () => {
					if (documentData) {
						documentData.content = editorContent;
						update_document(documentData);
					}
				});

				// Set documentReady to true after everything is loaded
				documentReady = true;

				// Set navbarReady after a delay to create staggered animation
				setTimeout(() => {
					navbarReady = true;
				}, 400); // Delay navbar animation to happen after document picker

				// Add this line to update line numbers when document loads
				updateLineNumbers();

				// Initial height adjustment
				setTimeout(adjustTextareaHeight, 0);
			} else {
				error = true;
			}
		} catch (e) {
			console.error('Error loading document:', e);
			error = true;
			throw e;
		}
	}

	// Function to load user profile data
	async function loadUserProfile() {
		try {
			// Get current user data
			const user = await get_current_user();
			if (user && user.id) {
				userId = user.id;
				
				// Try to load profile image with timestamp to prevent caching
				const timestamp = new Date().getTime();
				const imageUrl = `${get_profile_image_url(user.id)}?t=${timestamp}`;
				
				// Check if the image exists
				const response = await fetch(imageUrl, { method: 'HEAD' });
				if (response.ok) {
					userProfileImage = imageUrl;
				}
			}
		} catch (error) {
			console.error('Error loading user profile:', error);
		}
	}

	// Function to handle automatic line wrapping - fixed to prevent premature wrapping
	function autoWrapLine(text: string): string {
		if (!text) return '';
		
		// Split into existing lines first
		const lines = text.split('\n');
		const wrappedLines = [];
		
		for (const line of lines) {
			// If line is exactly at or under the limit, keep it as is
			if (line.length <= MAX_COLUMN_WIDTH) {
				wrappedLines.push(line);
			} else {
				// Line is too long and needs wrapping
				let remainingText = line;
				
				while (remainingText.length > MAX_COLUMN_WIDTH) {
					// Find the best place to break (prefer at spaces)
					// Only look for spaces up to MAX_COLUMN_WIDTH (not MAX_COLUMN_WIDTH-1)
					let breakIndex = remainingText.lastIndexOf(' ', MAX_COLUMN_WIDTH);
					
					// If no good break point or it's too far back, break exactly at the column limit
					if (breakIndex === -1 || breakIndex < MAX_COLUMN_WIDTH - 20) {
						breakIndex = MAX_COLUMN_WIDTH;
					}
					
					// Add the segment up to the break point
					wrappedLines.push(remainingText.substring(0, breakIndex));
					
					// Continue with the rest of the text, making sure to remove any leading spaces
					remainingText = remainingText.substring(breakIndex).trimStart();
				}
				
				// Add any remaining text as a new line (only if there's actually content)
				if (remainingText.length > 0) {
					wrappedLines.push(remainingText);
				}
			}
		}
		
		return wrappedLines.join('\n');
	}
	
	// Updated handleInput to prevent premature wrapping
	function handleInput(event: Event) {
		if (editorMode === 'NORMAL') {
			// Normal mode handling (unchanged)
			const selectionStart = editorElement.selectionStart;
			const selectionEnd = editorElement.selectionEnd;
			editorElement.value = editorContent;
			editorElement.setSelectionRange(selectionStart - 1, selectionEnd - 1);
			event.preventDefault();
		} else {
			// In INSERT mode
			const previousContent = editorContent;
			let currentContent = editorElement.value;
			const cursorPos = editorElement.selectionStart;
			
			// Check if wrapping is needed - only if a line is STRICTLY longer than the limit
			const contentLines = currentContent.split('\n');
			let needsWrapping = false;
			
			for (const line of contentLines) {
				if (line.length > MAX_COLUMN_WIDTH) {
					needsWrapping = true;
					break;
				}
			}
			
			// Apply wrapping if needed
			if (needsWrapping) {
				// Save text before cursor to calculate new cursor position later
				const beforeCursor = currentContent.substring(0, cursorPos);
				
				// Apply wrapping
				const wrappedContent = autoWrapLine(currentContent);
				
				// Only update if wrapping actually changed something
				if (wrappedContent !== currentContent) {
					// Update content
					editorContent = wrappedContent;
					editorElement.value = wrappedContent;
					
					// Recalculate cursor position
					const wrappedBeforeCursor = autoWrapLine(beforeCursor);
					const newCursorPos = wrappedBeforeCursor.length;
					
					// Set cursor position
					editorElement.setSelectionRange(newCursorPos, newCursorPos);
					
					// Update current content after wrapping
					currentContent = wrappedContent;
				} else {
					editorContent = currentContent;
				}
			} else {
				// No wrapping needed
				editorContent = currentContent;
			}
			
			// After all content changes, update line tracking
			const finalPosition = editorElement.selectionStart;
			const finalTextBeforeCursor = currentContent.substring(0, finalPosition);
			
			// Count actual newlines to determine line number
			const newlineMatches = finalTextBeforeCursor.match(/\n/g);
			const lineCount = newlineMatches ? newlineMatches.length + 1 : 1;
			
			// Update cursor line and active line index
			cursorLine = lineCount;
			activeLineIndex = lineCount - 1;
			
			// Update the lines array
			lines = currentContent.split('\n');
			
			// Update line numbers and adjust textarea height
			updateLineNumbers();
			updateCursorPosition(); // Make sure cursor position indicator is updated
			adjustTextareaHeight();
		}
	}
	
	// Update the paste handler to ensure consistent line handling
	function handlePaste(event: ClipboardEvent) {
		// Let the paste happen normally
		setTimeout(() => {
			// Get content after paste
			const pastedContent = editorElement.value;
			const cursorPos = editorElement.selectionStart;
			
			// Check if any line needs wrapping
			const contentLines = pastedContent.split('\n');
			let needsWrapping = false;
			
			for (const line of contentLines) {
				if (line.length > MAX_COLUMN_WIDTH) {
					needsWrapping = true;
					break;
				}
			}
			
			// Apply wrapping if needed
			if (needsWrapping) {
				// Get text before cursor to calculate new position later
				const beforeCursor = pastedContent.substring(0, cursorPos);
				
				// Apply wrapping
				const wrappedContent = autoWrapLine(pastedContent);
				
				// Update content
				editorContent = wrappedContent;
				editorElement.value = wrappedContent;
				
				// Calculate new cursor position
				const wrappedBeforeCursor = autoWrapLine(beforeCursor);
				const newCursorPos = wrappedBeforeCursor.length;
				
				// Set cursor position
				editorElement.setSelectionRange(newCursorPos, newCursorPos);
			} else {
				// No wrapping needed
				editorContent = pastedContent;
			}
			
			// Update lines array and cursor position
			lines = editorContent.split('\n');
			updateCursorPosition();
			updateLineNumbers();
			adjustTextareaHeight();
		}, 0);
	}
	
	// Add this function to measure column width
	function getCharacterWidth(): number {
		// This function creates a temporary span to measure character width
		// We're using a monospace font, so all characters have the same width
		const span = document.createElement('span');
		span.style.visibility = 'hidden';
		span.style.position = 'absolute';
		span.style.whiteSpace = 'nowrap';
		span.style.font = window.getComputedStyle(editorElement).font;
		span.innerHTML = 'X'.repeat(10); // Use 10 characters for more precise measurement
		
		document.body.appendChild(span);
		const width = span.getBoundingClientRect().width / 10;
		document.body.removeChild(span);
		
		return width;
	}
	
	// Calculate the max column width based on the editor width
	function calculateMaxColumnWidth(): number {
		if (!editorElement) return MAX_COLUMN_WIDTH;
		
		const charWidth = getCharacterWidth();
		const editorWidth = editorElement.clientWidth;
		const padding = parseInt(window.getComputedStyle(editorElement).paddingLeft) + 
					   parseInt(window.getComputedStyle(editorElement).paddingRight);
		
		// Calculate how many characters fit in the editor width
		const maxChars = Math.floor((editorWidth - padding) / charWidth);
		
		// Return the calculated value, or default if calculation fails
		return maxChars > 0 ? maxChars : MAX_COLUMN_WIDTH;
	}
	
	// Update onMount to calculate max column width when editor loads
	onMount(async () => {
		// Load document data and profile image in parallel
		try {
			const [docResult, userResult] = await Promise.all([
				loadDocumentData(),
				loadUserProfile()
			]);
			
			// Calculate max column width once the editor is loaded
			setTimeout(() => {
				if (editorElement) {
					const calculatedWidth = calculateMaxColumnWidth();
					// Only update if the calculation seems reasonable
					if (calculatedWidth > 20 && calculatedWidth < 200) {
						// We don't actually change MAX_COLUMN_WIDTH as it's a const
						// But we could use this value in a non-const variable if needed
						console.log(`Calculated max column width: ${calculatedWidth}`);
					}
				}
			}, 100);
			
		} catch (e) {
			console.error('Error during initialization:', e);
			error = true;
		} finally {
			loading = false;
		}
	});

	// Update the updateLineNumbers function to use cursor position directly
	function updateLineNumbers() {
		// Split content by newlines to get lines
		lines = editorContent.split('\n');
		
		// Ensure we have at least one line
		if (lines.length === 0) {
			lines = [''];
		}
		
		// Update line numbers display
		const lineNumbersContainer = document.querySelector('.line-numbers') as HTMLElement;
		if (lineNumbersContainer) {
			// Clear existing line numbers
			lineNumbersContainer.innerHTML = '';
			
			// Create new line numbers with correct highlighting
			lines.forEach((_, index) => {
				const lineNumberDiv = document.createElement('div');
				lineNumberDiv.className = `line-number ${index === activeLineIndex ? 'active' : ''}`;
				lineNumberDiv.textContent = (index + 1).toString();
				lineNumbersContainer.appendChild(lineNumberDiv);
			});
		}
	}

	// Handle cleanup in onDestroy instead
	onDestroy(() => {
		if (autoSaveCleanup) {
			autoSaveCleanup();
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

	// Add export to PDF function
	async function exportToPDF() {
		if (!documentData) return;
		
		try {
			// Create a new PDF document
			const doc = new jsPDF({
				orientation: 'portrait',
				unit: 'mm',
				format: 'a4'
			});
			
			// Set font and text properties
			doc.setFont('Helvetica');
			doc.setFontSize(16);
			
			// Add document title
			doc.text(documentData.name || 'Untitled Document', 20, 20);
			
			// Add line under title
			doc.setLineWidth(0.5);
			doc.line(20, 25, 190, 25);
			
			// Add creation date
			const createdAt = new Date(documentData.created_at).toLocaleDateString();
			doc.setFontSize(10);
			doc.text(`Created: ${createdAt}`, 20, 32);
			
			// Add content with line wrapping
			doc.setFontSize(12);
			const lines = editorContent.split('\n');
			let y = 40;
			const lineHeight = 7;
			
			for (const line of lines) {
				// Check if we need a new page
				if (y > 270) {
					doc.addPage();
					y = 20;
				}
				
				// Add the line to the PDF
				const splitLines = doc.splitTextToSize(line, 170);
				doc.text(splitLines, 20, y);
				y += splitLines.length * lineHeight;
			}
			
			// Save the PDF with the document name
			doc.save(`${documentData.name || 'document'}.pdf`);
		} catch (error) {
			console.error('Error exporting PDF:', error);
			alert('Failed to export PDF. Please try again.');
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
			
			<!-- Export to PDF button -->
			<button 
				class="btn btn-outline-light btn-sm me-3" 
				on:click={exportToPDF}
				title="Export to PDF"
				disabled={loading || error}
			>
				<i class="bi bi-file-earmark-pdf me-1"></i> Export PDF
			</button>
			
			<div class="dropdown">
				<button 
					class="btn p-0 border-0 bg-transparent" 
					data-bs-toggle="dropdown"
					aria-expanded="false"
					aria-haspopup="true"
					aria-label="Profile menu"
				>
					<img 
						src={userProfileImage} 
						alt="Profile" 
						class="rounded-circle profile-img"
						style="width: 40px; height: 40px; border: 2px solid var(--color-primary); margin-right: 10px; object-fit: cover;"
						on:error={() => userProfileImage = profileDefault}
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
			<button class="commands-toggle" on:click={() => showCommands = !showCommands} title="Toggle Commands Reference">
				<i class="bi bi-info-circle"></i>
			</button>
			<span>Line: {cursorLine}, Col: {cursorColumn}</span>
		</div>
	</div>

	<!-- Add commands cheat sheet overlay -->
	<div class="commands-overlay" class:show-commands={showCommands}>
		<div class="commands-header">
			<h5>Vim Command Reference</h5>
			<button class="commands-close" on:click={() => showCommands = false}>×</button>
		</div>
		<div class="commands-body">
			<div class="commands-section">
				<h6>Mode Switching</h6>
				<ul>
					<li><span class="key">i</span> Enter Insert mode</li>
					<li><span class="key">Esc</span> Return to Normal mode</li>
					<li><span class="key">:</span> Enter Command mode</li>
				</ul>
			</div>
			
			<div class="commands-section">
				<h6>Navigation</h6>
				<ul>
					<li><span class="key">h j k l</span> Move left, down, up, right</li>
					<li><span class="key">0</span> Start of line</li>
					<li><span class="key">$</span> End of line</li>
					<li><span class="key">gg</span> Start of document</li>
					<li><span class="key">G</span> End of document</li>
				</ul>
			</div>
			
			<div class="commands-section">
				<h6>Editing</h6>
				<ul>
					<li><span class="key">x</span> Delete character</li>
					<li><span class="key">dd</span> Delete line</li>
					<li><span class="key">yy</span> Copy line</li>
					<li><span class="key">p</span> Paste after cursor</li>
					<li><span class="key">P</span> Paste before cursor</li>
					<li><span class="key">u</span> Undo</li>
					<li><span class="key">Ctrl+r</span> Redo</li>
				</ul>
			</div>
			
			<div class="commands-section">
				<h6>Search & Replace</h6>
				<ul>
					<li><span class="key">/pattern</span> Search forward</li>
					<li><span class="key">?pattern</span> Search backward</li>
					<li><span class="key">n</span> Next match</li>
					<li><span class="key">N</span> Previous match</li>
					<li><span class="key">:%s/old/new/g</span> Replace all</li>
				</ul>
			</div>
		</div>
	</div>
</div>

<style>
.editor-page {
    position: relative;
    width: 100%;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    background: transparent;
}

.background-image {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    z-index: -1;
    background-size: cover;
    background-position: center;
    opacity: 0.15;
    pointer-events: none;
}

.navbar-container {
    position: sticky;
    top: 0;
    z-index: 100;
    background-color: rgba(var(--color-background-rgb), 0.95);
    backdrop-filter: blur(5px);
    border-bottom: none;
}

.editor-container {
    flex: 1;
    width: 100%;
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
    padding-bottom: calc(4rem + 60px); /* Increased bottom padding significantly */
    position: relative;
    display: flex;
    flex-direction: column;
    background: transparent;
}

.editor-wrapper {
    position: relative;
    width: 100%;
    background-color: rgba(40, 40, 40, 0.3);
    border-radius: 8px;
    box-shadow: 0 0 10px rgba(0, 0, 0, 0.2);
    backdrop-filter: blur(10px);
    display: flex;
    flex-direction: column;
    min-height: 100%;
    margin-bottom: 40px; /* Increased margin at bottom */
}

.editor-content {
    display: flex;
    width: 100%;
    position: relative;
    background: transparent;
    border-radius: 8px;
    min-height: 100%;
}

.line-numbers {
    position: sticky;
    left: 0;
    padding-top: 0.5rem;
    padding-bottom: 3rem; /* Match textarea padding */
    background-color: rgba(40, 40, 40, 0.3);
    border-right: 1px solid var(--color-border);
    z-index: 1;
    backdrop-filter: blur(5px);
    width: 3.5rem;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 100%;
}

.editor-textarea {
    flex: 1;
    width: 100%;
    border: none;
    background: transparent;
    color: var(--color-text);
    font-family: 'JetBrains Mono', monospace;
    font-size: 1rem;
    line-height: 1.5rem;
    padding: 0.5rem 1rem;
    padding-bottom: 3rem; /* Increased bottom padding */
    resize: none;
    outline: none;
    min-height: 100%;
}

.line-number {
    color: var(--color-text-muted);
    padding: 0 0.5rem;
    text-align: right;
    user-select: none;
    line-height: 1.5rem;
    width: 100%;
    font-variant-numeric: tabular-nums;
    height: 1.5rem; /* Explicit height for each line number */
}

.line-number.active {
    color: var(--color-primary);
    background-color: rgba(var(--color-primary-rgb), 0.1);
}

.status-bar {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    height: 2rem;
    background-color: rgba(var(--color-background-rgb), 0.95);
    border-top: 1px solid var(--color-border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 1rem;
    z-index: 100;
    backdrop-filter: blur(5px);
}

/* Animation classes */
.fade-in {
    opacity: 0;
    animation: fadeIn 0.3s ease-in forwards;
}

@keyframes fadeIn {
    from { 
        opacity: 0;
        transform: translateY(-10px);
    }
    to { 
        opacity: 1;
        transform: translateY(0);
    }
}

/* Document switcher styles */
.document-switcher {
    display: flex;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.5rem;
    margin-top: 0.5rem;
    background-color: rgba(var(--color-background-rgb), 0.95);
    backdrop-filter: blur(5px);
    opacity: 0;
    animation: fadeIn 0.3s ease-in forwards 0.2s;
}

.doc-button {
    padding: 0.25rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: transparent;
    color: var(--color-text);
    cursor: pointer;
    transition: all 0.2s ease;
}

.doc-button:hover:not(:disabled) {
    background-color: rgba(var(--color-primary-rgb), 0.1);
}

.doc-button.active {
    background-color: var(--color-primary);
    color: white;
    border-color: var(--color-primary);
}

.doc-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

/* Add these styles for the commands overlay */
.commands-overlay {
    position: fixed;
    top: 100px;
    right: 20px;
    width: 300px;
    background-color: rgba(18, 18, 18, 0.9);
    border: 1px solid rgba(16, 185, 129, 0.5);
    border-radius: 8px;
    box-shadow: 0 4px 15px rgba(0, 0, 0, 0.3);
    color: #e5e5e5;
    z-index: 1000;
    font-family: 'JetBrains Mono', monospace;
    font-size: 12px;
    backdrop-filter: blur(5px);
    opacity: 0;
    transform: translateX(20px);
    pointer-events: none;
    transition: all 0.3s ease;
    max-height: 70vh;
    overflow-y: auto;
}

.commands-overlay.show-commands {
    opacity: 1;
    transform: translateX(0);
    pointer-events: all;
}

.commands-header {
    padding: 10px 15px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.commands-header h5 {
    margin: 0;
    font-size: 14px;
    color: var(--color-primary);
}

.commands-close {
    background: none;
    border: none;
    color: #999;
    font-size: 20px;
    cursor: pointer;
    padding: 0;
    margin-left: 10px;
}

.commands-close:hover {
    color: #fff;
}

.commands-body {
    padding: 10px 15px;
}

.commands-section {
    margin-bottom: 15px;
}

.commands-section h6 {
    margin-bottom: 8px;
    color: var(--color-primary);
    font-size: 13px;
}

.commands-section ul {
    list-style: none;
    padding: 0;
    margin: 0;
}

.commands-section li {
    margin-bottom: 5px;
    display: flex;
    align-items: center;
}

.key {
    display: inline-block;
    padding: 2px 5px;
    background-color: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
    margin-right: 8px;
    font-weight: bold;
    color: var(--color-primary-light);
}

.commands-toggle {
    background: none;
    border: none;
    color: #999;
    cursor: pointer;
    margin-right: 10px;
    padding: 2px 5px;
    border-radius: 3px;
    transition: all 0.2s ease;
}

.commands-toggle:hover {
    color: var(--color-primary);
    background-color: rgba(255, 255, 255, 0.1);
}

/* Make sure these styles don't conflict with existing ones */
.cursor-position {
    display: flex;
    align-items: center;
}
</style>
