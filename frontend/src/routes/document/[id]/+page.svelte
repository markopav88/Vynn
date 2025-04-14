<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { jsPDF } from 'jspdf';
	import { browser } from '$app/environment';
	import { get_document, update_document, setup_auto_save, get_project_from_document } from '$lib/ts/document';
	import { logout, get_current_user, get_profile_image_url } from '$lib/ts/user'
	import { get_project_documents } from '$lib/ts/project';
	import { handleNormalModeKeydown } from '$lib/ts/editor-commands';
	import Toast from '$lib/components/Toast.svelte';

	import logo from '$lib/assets/logo.png';
	import backgroundImage from '$lib/assets/editor-background.jpg';
	import profileDefault from '$lib/assets/profile-image.png';

	import '$lib/assets/style/document.css'

	// Define a type that can be either HTMLDivElement or HTMLTextAreaElement
	type EditorElement = HTMLDivElement;

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
	let editorMode: 'NORMAL' | 'INSERT' | 'COMMAND' = 'NORMAL'; // Initialize to NORMAL mode
	let cursorLine = 1; // for indicator in bottom right
	let cursorColumn = 1; // for indicator in bottom right
	let editorElement: HTMLDivElement | null = null;

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
	let commandInputElement: HTMLInputElement | null = null;
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

	// Initialize editor element

	let commandMode = false;

	// Add at the top of the script where the other variable declarations are
	let lastColumnPerLine: number[] = [];

	// Add these two helper functions
	function updateLastColumnForCurrentLine() {
		// Ensure the array has enough entries
		while (lastColumnPerLine.length < lines.length) {
			lastColumnPerLine.push(0);
		}
		
		// Get current cursor position
		const selection = window.getSelection();
		if (!selection || !selection.rangeCount) return;
		
		const range = selection.getRangeAt(0);
		const currentOffset = getTextOffset(range.startContainer, range.startOffset);
		const textBeforeCursor = editorContent.substring(0, currentOffset);
		const linesBeforeCursor = textBeforeCursor.split('\n');
		const currentColumn = linesBeforeCursor[linesBeforeCursor.length - 1].length;
		
		// Save current column for this line
		lastColumnPerLine[activeLineIndex] = currentColumn;
	}

	function getSavedColumnForLine(lineIndex: number): number {
		if (lineIndex >= 0 && lineIndex < lastColumnPerLine.length) {
			return lastColumnPerLine[lineIndex];
		}
		return 0;
	}
	// Add a function to prevent default browser behavior for certain key combinations
	function preventBrowserDefaults(event: KeyboardEvent) {
		// Prevent OS shortcuts by capturing all Ctrl/Cmd combinations
		if (event.ctrlKey || event.metaKey) {
			// Allow only specific browser shortcuts we want to keep
			const allowedKeys = ['c', 'v', 'a', 'z', 'y', 'f', '/'];
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
		if (!browser) return;
		
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
			if (documentData && editorElement) {
				console.log('Saving current document before switching');
				documentData.content = editorElement.innerHTML;
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
				
				// Set the HTML content in the editor element
				if (editorElement) {
					editorElement.innerHTML = editorContent;
					
					// Update line numbers and cursor position
					setTimeout(() => {
						updateLineNumbers();
						updateCursorPosition();
						adjustEditorHeight();
					}, 10);
				}
				
				// Set the HTML content in the editor element
				if (editorElement) {
					// Set HTML content
					editorElement.innerHTML = editorContent;
					
					// Count actual divs in the content for line numbering
					const divCount = editorElement.querySelectorAll('div').length;
					console.log(`Document switched: found ${divCount} divs for line numbering`);
					
					// If content has no divs but has content, convert it to proper structure
					if (divCount === 0 && editorContent.trim() !== '') {
						// Split by newlines and create proper div structure
						const lines = editorContent.split('\n');
						console.log(`Converting content to ${lines.length} divs`);
						
						// Create HTML with divs
						const htmlWithDivs = lines.map(line => 
							line.trim() === '' ? '<div><br></div>' : `<div>${line}</div>`
						).join('');
						
						// Set the properly formatted HTML
						editorElement.innerHTML = htmlWithDivs;
					} else if (divCount === 0) {
						// Empty document case - add at least one empty div
						editorElement.innerHTML = '<div><br></div>';
						console.log("Empty document: adding empty div");
					}
					
					
					// First update immediately
					updateLineNumbers();
					
					// Then update after a short delay to ensure the DOM has stabilized
					setTimeout(() => {
						console.log("Refreshing line numbers after document switch");
						// Force a complete refresh of the line numbers based on div count
						const finalDivCount = editorElement?.querySelectorAll('div').length ?? 0;
						console.log(`Document switch complete: ${finalDivCount} lines detected`);
						
						// Make sure we're using the right line count by forcing a refresh
						updateLineNumbers();
						updateCursorPosition();
						adjustEditorHeight();
						// Set cursor to beginning of document
						if (editorElement && editorElement.firstChild) {
							const range = document.createRange();
							range.setStart(editorElement.firstChild, 0);
							range.collapse(true);
							const selection = window.getSelection();
							if (selection) {
								selection.removeAllRanges();
								selection.addRange(range);
							}
						}
					}, 50);

					// Then update after a short delay to ensure the DOM has stabilized
					setTimeout(() => {
						if (editorElement) {
							console.log("Refreshing line numbers after document switch");
							// Force a complete refresh of the line numbers based on div count
							const finalDivCount = editorElement.querySelectorAll('div').length;
							console.log(`Document switch complete: ${finalDivCount} lines detected`);
							
							// Make sure we're using the right line count by forcing a refresh
							updateLineNumbers();
							updateCursorPosition();
							adjustEditorHeight();
							
							// Set cursor to beginning of document
							if (editorElement.firstChild) {
								const range = document.createRange();
								range.setStart(editorElement.firstChild, 0);
								range.collapse(true);
								
								const selection = window.getSelection();
								if (selection) {
									selection.removeAllRanges();
									selection.addRange(range);
								}
							}
						}
					}, 50);
				}
				
				// Update current document index
				currentDocumentIndex = projectDocuments.findIndex((doc) => doc.id === docId);

				// Get project info for the new document to ensure project name is correct
				const projectInfo = await get_project_from_document(docId);
				if (projectInfo && projectInfo.project_id) {
					documentData.project_name = projectInfo.project_name;
				}

				// Wait for animation to complete before resetting animation state
				setTimeout(() => {
					isAnimating = false;
					slideDirection = '';
					previousDocumentContent = '';
					previousDocumentLines = [];
					animationHeight = 0;
					// Adjust textarea height
					setTimeout(adjustEditorHeight, 0);
				}, 400); // Match this with the CSS animation duration (400ms now)

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
				setRange(editorElement, position, position + commandInput.length);

				// Ensure the cursor is visible
				const textBeforeCursor = editorContent.substring(0, position);
				const lines = textBeforeCursor.split('\n');
				activeLineIndex = lines.length - 1;

				// Update cursor position
				cursorLine = lines.length;
				cursorColumn = lines[lines.length - 1].length + 1;
				
				// Make sure the cursor is visible by scrolling
				ensureCursorVisible();
			}
		}
	}

	// Function to navigate through search results with n/m
	function navigateSearchResults(forward: boolean) {
		if (searchResults.length === 0) return;

		// Get search direction (whether we're in a forward or backward search)
		const isBackwardSearch = commandPrefix === '?';
		
		// Determine which direction to move based on search direction and key pressed
		// For '/' searches: 'n' moves forward, 'm' moves backward
		// For '?' searches: 'n' moves backward, 'm' moves forward
		let moveForward = forward;
		
		// If we're in a backward search ('?'), invert the direction
		if (isBackwardSearch) {
			moveForward = !moveForward;
		}
		
		if (moveForward) {
			currentSearchIndex = (currentSearchIndex + 1) % searchResults.length;
		} else {
			currentSearchIndex = (currentSearchIndex - 1 + searchResults.length) % searchResults.length;
		}

		navigateToSearchResult();
		
		// Show feedback about current match position
		showCommandError(`Match ${currentSearchIndex + 1} of ${searchResults.length}`);
	}

	// Add toast types and state
	type ToastData = {
		message: string;
		type: 'success' | 'error' | 'warning';
	};
	
	let toasts: ToastData[] = [];
	
	// Function to show a toast notification
	function showToast(message: string, type: 'success' | 'error' | 'warning' = 'success') {
		toasts = [...toasts, { message, type }];
		// Remove the toast after 3 seconds
		setTimeout(() => {
			toasts = toasts.filter(t => t.message !== message);
		}, 3000);
	}
	
	// Function to remove a toast
	function removeToast(index: number) {
		toasts = toasts.filter((_, i) => i !== index);
	}

	// Function to handle colon commands
	function handleColonCommand(command: string) {
		// Simple command handling for now
		const cmd = command.trim().toLowerCase();

		if (cmd === 'q' || cmd === 'quit') {
			// Navigate back to drive
			goto('/drive');
			return true;
		} else if (cmd === 'w' || cmd === 'write') {
			// Save the document
			if (documentData && editorElement) {
				// Get the current content from the editor
				documentData.content = editorElement.innerHTML;
				// Save it
				update_document(documentData).then(() => {
					showToast('Document saved successfully', 'success');
				}).catch((error) => {
					console.error('Error saving document:', error);
					showToast('Failed to save document', 'error');
				});
				return true;
			}
		} else if (cmd === 'wq') {
			// Save and quit
			if (documentData && editorElement) {
				// Get the current content
				documentData.content = editorElement.innerHTML;
				// Save and then navigate
				update_document(documentData).then(() => {
					showToast('Document saved successfully', 'success');
					goto('/drive');
				}).catch((error) => {
					console.error('Error saving document:', error);
					showToast('Failed to save document', 'error');
				});
				return true;
			}
		} else if (cmd === 'export') {
			// Export document to PDF
			exportToPDF();
			return true;
		} else if (cmd.startsWith('%s/')) {
			// Handle find and replace command
			const parts = cmd.split('/');
			if (parts.length >= 3) {
				const searchText = parts[1];
				const replaceText = parts[2];
				const flags = parts.length > 3 ? parts[3] : '';
				const isGlobal = flags.includes('g');
				const isCaseInsensitive = flags.includes('i');

				if (searchText && replaceText) {
					// Create a regular expression for the search with proper flags
					const regexFlags = (isGlobal ? 'g' : '') + (isCaseInsensitive ? 'i' : '');
					const searchRegex = new RegExp(searchText, regexFlags);
					
					// Perform the replacement
					const newContent = editorContent.replace(searchRegex, replaceText);
					
					// Update the editor content
					editorContent = newContent;
					
					// Update the document data
					if (documentData) {
						documentData.content = newContent;
						update_document(documentData);
					}
					
					// Show success message
					const replacementCount = (editorContent.match(searchRegex) || []).length;
					showCommandError(`Replaced ${replacementCount} occurrence${replacementCount !== 1 ? 's' : ''} of "${searchText}" with "${replaceText}"`);
				} else {
					showCommandError('Invalid find and replace syntax. Use :%s/search/replace/gi for global case-insensitive replace');
				}
			} else {
				showCommandError('Invalid find and replace syntax. Use :%s/search/replace/gi for global case-insensitive replace');
			}
			return true;
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
		console.log("Buffer:", normalModeBuffer);

		// Clear any existing timeout
		if (normalModeBufferTimeout) {
			clearTimeout(normalModeBufferTimeout);
		}

		// Set a timeout to clear the buffer after a delay
		normalModeBufferTimeout = setTimeout(() => {
			// Important: make sure we don't trigger any content changes by the buffer clearing
			const oldBuffer = normalModeBuffer;
			normalModeBuffer = '';
			console.log(`Buffer "${oldBuffer}" cleared by timeout without taking action`);
			
			// Update cursor position and line numbers to reflect current state
			if (editorElement) {
				// Safely update position without changing content
				updateCursorPosition();
				updateLineNumbers();
			}
		}, 800); // 800ms timeout for multi-key commands

		// Explicitly log and return false for incomplete sequences
		if (normalModeBuffer === 'g' || normalModeBuffer === 'd' || normalModeBuffer === 'y') {
			console.log(`Incomplete command: '${normalModeBuffer}' - waiting for more input`);
			return false;
		}

		// Only take action if we have a complete command sequence
		// Check for complete sequences only - don't do anything for partial commands
		if (normalModeBuffer === 'yy') {
			// Copy the current line or selection
			copyText();
			normalModeBuffer = ''; // Clear buffer after command
			return true;
		} else if (normalModeBuffer === 'dd') {
			console.log("Executing 'dd' command - deleting current line");
			// Delete the current line where the cursor is
			deleteCurrentLine();
			normalModeBuffer = ''; // Clear buffer after command
			return true;
		} else if (normalModeBuffer === 'gg') {
			// Move cursor to first line, first position without modifying structure
			if (editorElement && editorContent) {
				try {
					// Get the first div or the editor itself
					const firstDiv = editorElement.querySelector('div') || editorElement;
					
					// Create a range at the start of the first div
					const range = document.createRange();
					
					// Set the range at position 0 properly
					if (firstDiv.firstChild && firstDiv.firstChild.nodeType === Node.TEXT_NODE) {
						range.setStart(firstDiv.firstChild, 0);
					} else {
						range.setStart(firstDiv, 0);
					}
					range.collapse(true);
					
					// Apply the range for cursor placement
					const selection = window.getSelection();
					if (selection) {
						editorElement.focus(); // Make sure editor is focused
						selection.removeAllRanges();
						selection.addRange(range);
					}
					
					// Update tracking variables
					activeLineIndex = 0;
					cursorLine = 1;
					cursorColumn = 1;
					
					// Update UI without changing content or structure
					updateCursorPosition();
					updateLineNumbers();
					
					// Debug
					console.log('gg command executed: cursor at first line without content changes');
				} catch (error) {
					console.error("Error in gg command:", error);
				}
			}
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

		// Use Selection API instead of selectionStart/End
		const selection = window.getSelection();
		if (selection && selection.rangeCount > 0 && !selection.isCollapsed) {
			// Copy selected text
			const range = selection.getRangeAt(0);
			const start = getTextOffset(range.startContainer, range.startOffset);
			const end = getTextOffset(range.endContainer, range.endOffset);
			textToCopy = editorContent.substring(start, end);
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

	// Function to delete text
	function deleteText() {
		if (!editorElement) return;

		// Check if there's a selection
		const selection = window.getSelection();
		if (!selection || selection.rangeCount === 0) return;
		
		const range = selection.getRangeAt(0);
		
		// Handle normal text selection deletion (works across divs)
		if (!range.collapsed) {
			// Store the start position for cursor restoration
			const start = getTextOffset(range.startContainer, range.startOffset);
			
			// Use execCommand for the actual deletion which properly handles multi-line cases
			document.execCommand('delete', false);
			
			// After deleting, check for and remove empty divs
			const currentDivs = Array.from(editorElement.querySelectorAll('div'));
			let emptyDivs = currentDivs.filter(div => (div.textContent || '').trim() === '');
			
			// Only remove empty divs if they're not the only div
			if (emptyDivs.length > 0 && emptyDivs.length < currentDivs.length) {
				emptyDivs.forEach(div => {
					div.remove();
				});
			}
			
			// Get updated content after removal
			editorContent = getEditorContent();
			
			// Try to restore a reasonable cursor position
			const newContentLength = editorContent.length;
			const safePosition = Math.min(start, newContentLength);
			setRange(editorElement, safePosition, safePosition);
		}
		else {
			// If no selection but cursor is in a div, try to delete current character
			const currentNode = range.startContainer;
			const offset = range.startOffset;
			const currentOffset = getTextOffset(currentNode, offset);
			
			// Only delete if we're not at the end of the document
			if (currentOffset < editorContent.length) {
				// Delete one character at cursor position
				editorContent = editorContent.substring(0, currentOffset) + editorContent.substring(currentOffset + 1);
				
				// Update the editor using our safe method
				safelySetEditorContent(editorContent);
				
				// Check for divs that became empty
				const allDivs = Array.from(editorElement.querySelectorAll('div'));
				const emptyDivs = allDivs.filter(div => (div.textContent || '').trim() === '');
				
				// Remove empty divs if there are other non-empty divs
				if (emptyDivs.length > 0 && emptyDivs.length < allDivs.length) {
					emptyDivs.forEach(div => {
						div.remove();
					});
					
					// Get updated content after removing empty divs
					editorContent = getEditorContent();
				}
				
				// Restore cursor position
				setRange(editorElement, currentOffset, currentOffset);
			}
		}
		
		// Update lines array for line numbers
		lines = editorContent.split('\n');
		
		// Update UI
		updateCursorPosition();
		updateLineNumbers();
		adjustEditorHeight();
	}

	// Function to paste text
	function pasteText() {
		if (!editorElement || !clipboardText) return;

		// Get the cursor position from selection
		const selection = window.getSelection();
		if (!selection || !selection.rangeCount) return;
		
		const range = selection.getRangeAt(0);
		const start = getTextOffset(range.startContainer, range.startOffset);
		const end = getTextOffset(range.endContainer, range.endOffset);

		// Insert the clipboard text at the current cursor position
		const beforePaste = editorContent.substring(0, start);
		const afterPaste = editorContent.substring(end);
		const newContent = beforePaste + clipboardText + afterPaste;
		editorContent = newContent;
		
		// Update the editor content using our safe method
		safelySetEditorContent(newContent);
		
		// Set cursor position after the pasted text
		if (editorElement) {
			setRange(editorElement, start + clipboardText.length, start + clipboardText.length);
		}
		
		// Update line numbers and other UI elements
		lines = editorContent.split('\n');
		updateCursorPosition();
		adjustEditorHeight();
	}

	// Function to delete the current line - update to adjust height after deletion
	function deleteCurrentLine() {
		if (!editorElement) return;

		console.log(`Deleting line at index ${activeLineIndex}`);

		// For contenteditable, get all divs and delete the one at the active index
		const allDivs = Array.from(editorElement.querySelectorAll('div'));
		
		// Make sure we have a valid index
		if (activeLineIndex >= 0 && activeLineIndex < allDivs.length) {
			// Get the div to remove
			const divToRemove = allDivs[activeLineIndex];
			
			// Remove the div directly from the DOM 
			divToRemove.remove();
			
			// If we removed all divs, add an empty one to maintain editor structure
			if (allDivs.length === 1) {
				const emptyDiv = document.createElement('div');
				emptyDiv.appendChild(document.createTextNode('\u200B')); // Zero-width space
				editorElement.appendChild(emptyDiv);
			}
			
			// Update editor content based on current DOM structure
			editorContent = getEditorContent();
			
			// Calculate new cursor position - move to beginning of the same line, or line above if last line
			const newLineIndex = Math.min(activeLineIndex, allDivs.length - 2);
			activeLineIndex = Math.max(0, newLineIndex);
			
			// Position cursor at the beginning of the line
			if (allDivs.length > 1) {
				const targetDiv = editorElement.querySelectorAll('div')[activeLineIndex];
				if (targetDiv) {
					const range = document.createRange();
					if (targetDiv.firstChild && targetDiv.firstChild.nodeType === Node.TEXT_NODE) {
						range.setStart(targetDiv.firstChild, 0);
					} else {
						range.setStart(targetDiv, 0);
					}
					range.collapse(true);
					
					const selection = window.getSelection();
					if (selection) {
						selection.removeAllRanges();
						selection.addRange(range);
					}
				}
			}
		}
		
		// Update lines array for line numbers
		lines = editorContent.split('\n');
		
		// Update UI
		updateCursorPosition();
		updateLineNumbers();
		adjustEditorHeight();
		
		// Show feedback
		showCommandError('Line deleted');
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

	// Special handler for Enter key to fix line counting issues
	function handleKeyDown(event: KeyboardEvent) {
		if (!event) return;
		
		preventBrowserDefaults(event);

		// Handle Ctrl+/ for toggling commands cheat sheet
		if (event.ctrlKey && (event.key === '/' || event.key === '?')) {
			event.preventDefault();
			showCommands = !showCommands;
			return;
		}

		// Handle document switching shortcuts (Ctrl + 1-9)
		if (event.ctrlKey && !event.altKey && !event.metaKey && !event.shiftKey) {
			// Check if the key is a number from 1-9
			const num = parseInt(event.key);
			if (!isNaN(num) && num >= 1 && num <= 9) {
				event.preventDefault();
				// Check if we have a document at this index
				if (projectDocuments && projectDocuments[num - 1]) {
					const targetDoc = projectDocuments[num - 1];
					switchDocument(targetDoc.id);
					return;
				}
			}
		}

		// Handle Escape key to exit any mode and return to NORMAL mode
		if (event.key === 'Escape') {
			editorMode = 'NORMAL';
			event.preventDefault();
			return;
		}

		// Rest of the existing handleKeyDown function...
		// ... existing code ...

		// COMMAND MODE: Handle only Escape (already done) and Enter
		if (editorMode === 'COMMAND') {
			if (event.key === 'Enter') {
				// Command execution is handled by the executeCommand function
				// which is bound directly to the command input
				return;
			}
			// Ignore all other keys in command mode
			return;
		}

		// INSERT MODE: Allow typing but handle minimal key commands
		if (editorMode === 'INSERT') {
			// Handle arrow keys in any mode - just update line highlighting
			if (event.key === 'ArrowUp' || event.key === 'ArrowDown' || 
				event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
				// Let browser handle actual cursor movement
				// Then update our position tracking
				setTimeout(() => {
					updateCursorPosition();
					updateLineNumbers();
				}, 0);
			}
			
			// Handle arrow keys in any mode - update line highlighting with better timing
			if (event.key === 'ArrowUp' || event.key === 'ArrowDown' || 
				event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
				// Let browser handle actual cursor movement
				// Then update our position tracking with two phases to ensure accuracy
				setTimeout(() => {
					// First update the cursor position
					updateCursorPosition();
					// Then update line numbers based on the updated cursor position
					updateLineNumbers();
					
					// Do a second update after a short delay to ensure accuracy
					setTimeout(() => {
						updateCursorPosition();
						updateLineNumbers();
					}, 10);
				}, 0);
			}
			
			// For all other keys in INSERT mode, just let the default behavior occur
			// and update the cursor position afterward
			setTimeout(() => {
				updateCursorPosition();
				updateLineNumbers();
				ensureCursorVisible();
			}, 0);
			
			return; // End INSERT mode handling
		}

		// NORMAL MODE: Handle all editor commands
		if (editorMode === 'NORMAL') {
			// Handle arrow keys
			if (event.key === 'ArrowUp' || event.key === 'ArrowDown' || 
				event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
					setTimeout(() => {
					// First update the cursor position
					updateCursorPosition();
					// Then update line numbers based on the updated cursor position
					updateLineNumbers();
					
					// Second phase update to ensure everything is in sync
					setTimeout(() => {
						updateCursorPosition();
						updateLineNumbers();
					}, 10);
				}, 0);
				return;
			}

			// Handle arrow keys with improved line number updating
			if (event.key === 'ArrowUp' || event.key === 'ArrowDown' || 
				event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
				// Let browser handle actual cursor movement
				// Then update with a multi-phase approach for better accuracy
				setTimeout(() => {
					// First update the cursor position
					updateCursorPosition();
					// Then update line numbers based on the updated cursor position
					updateLineNumbers();
					
					// Second phase update to ensure everything is in sync
					setTimeout(() => {
						updateCursorPosition();
						updateLineNumbers();
					}, 10);
				}, 0);
				return;
			}

			// List of allowed navigation keys that shouldn't be prevented
			const allowedKeys = [
				'Home',
				'End',
				'PageUp',
				'PageDown',
				'Tab'
			];

			// Block most key inputs in normal mode except for command keys
			const commandKeys = [':', '/', '?', 'i', 'x', 'y', 'p', 'd', 'n', 'm', 'h', 'j', 'k', 'l', 'u', '0', '$', 'g', 'G'];
			
			if (!event.ctrlKey && !allowedKeys.includes(event.key) && !commandKeys.includes(event.key)) {
				event.preventDefault();
			}

			// Handle Ctrl+r for redo
			if (event.ctrlKey && event.key === 'r') {
				event.preventDefault();
				performRedo();
				return;
			}

			// Vim-style movement keys
			if (event.key === 'h') {
				// Move left using native Selection API
				const selection = window.getSelection();
				if (selection && selection.rangeCount > 0) {
					const range = selection.getRangeAt(0);
					const currentOffset = getTextOffset(range.startContainer, range.startOffset);
					if (currentOffset > 0) {
						setCursorPositionByOffset(currentOffset - 1);
					}
				}
				setTimeout(() => {
					updateCursorPosition();
					updateLineNumbers();
				}, 0);
				event.preventDefault();
				return;
			} else if (event.key === 'l') {
				// Move right using native Selection API
				const selection = window.getSelection();
				if (selection && selection.rangeCount > 0) {
					const range = selection.getRangeAt(0);
					const currentOffset = getTextOffset(range.startContainer, range.startOffset);
					if (currentOffset < editorContent.length) {
						setCursorPositionByOffset(currentOffset + 1);
					}
				}
				setTimeout(() => {
					updateCursorPosition();
					updateLineNumbers();
				}, 0);
				event.preventDefault();
				return;
			} else if (event.key === 'k') {
				// Move up one line
				document.getSelection()?.modify("move", "backward", "line");
				setTimeout(() => {
					updateCursorPosition();
					updateLineNumbers();
				}, 0);
				event.preventDefault();
				return;
			} else if (event.key === 'j') {
				// Move down one line
				document.getSelection()?.modify("move", "forward", "line");
				setTimeout(() => {
					updateCursorPosition();
					updateLineNumbers();
				}, 0);
				event.preventDefault();
				return;
			}
          
			// Mode switching commands
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
			} 
			
			// Editing commands
			else if (event.key === 'x') {
				deleteText();
				event.preventDefault();
				return;
			} else if (event.key === 'p') {
				pasteText();
				ensureCursorVisible();
				event.preventDefault();
				return;
			} 
			
			// Search navigation
			else if (event.key === 'n' && searchResults.length > 0) {
				navigateSearchResults(true);
				ensureCursorVisible();
				event.preventDefault();
				return;
			} else if (event.key === 'm' && searchResults.length > 0) {
				navigateSearchResults(false);
				ensureCursorVisible();
				event.preventDefault();
				return;
			} 
			
			// Undo/Redo
			else if (event.key === 'u') {
				performUndo();
				ensureCursorVisible();
				event.preventDefault();
				return;
			} 
			
			// Line navigation
			else if (event.key === '0') {
				moveToStartOfLine();
				ensureCursorVisible();
				event.preventDefault();
				return;
			} else if (event.key === '$') {
				moveToEndOfLine();
				ensureCursorVisible();
				event.preventDefault();
				return;
			} 
			
			// Document navigation
			else if (event.key === 'G') {
				moveToEndOfDocument();
				ensureCursorVisible();
				event.preventDefault();
				return;
			} 
			
			// Multi-key commands
			else if (event.key === 'g') {
				// 'g' could be the start of 'gg' command
				event.preventDefault();
				handleNormalModeSequence(event.key);
				return;
			} else if (event.key === 'd' || event.key === 'y') {
				// For 'd' or 'y', just add to buffer
				event.preventDefault();
				handleNormalModeSequence(event.key);
				return;
			}
			
			// For all other keys in NORMAL mode, update UI but don't modify content
			setTimeout(() => {
				updateCursorPosition();
				updateLineNumbers();
				ensureCursorVisible();
			}, 0);
		}
	}

	// Update the updateCursorPosition function to ensure all lines (including empty ones) are properly highlighted
	function updateCursorPosition() {
		if (!editorElement || !browser) return;
		
		console.log("Updating cursor position");
		
		try {
			const selection = window.getSelection();
			if (!selection || selection.rangeCount === 0) return;
			
			const range = selection.getRangeAt(0);
			const textNode = range.startContainer;
			const offset = range.startOffset;
			
			// Get all divs first for reference
			const allDivs = Array.from(editorElement.querySelectorAll('div'));
			console.log(`Document has ${allDivs.length} divs`);
			
			// For each div element in the editor, check if it contains the cursor
			let foundInDivs = false;
			for (let i = 0; i < allDivs.length; i++) {
				const div = allDivs[i];
				if (div.contains(textNode) || 
				    (textNode === editorElement && offset >= i && offset <= i+1)) {
					// Found the div containing our cursor
					activeLineIndex = i;
					cursorLine = i + 1;
					foundInDivs = true;
					console.log(`Cursor found in div at index ${i}, setting activeLineIndex to ${i}`);
					
					// Calculate cursor column
					if (textNode.nodeType === Node.TEXT_NODE) {
						cursorColumn = offset + 1;
						if (textNode !== div.firstChild) {
							// Add length of any previous text nodes in the same div
							let node = div.firstChild;
							while (node && node !== textNode) {
								if (node.nodeType === Node.TEXT_NODE) {
									cursorColumn += node.textContent?.length || 0;
								}
								node = node.nextSibling;
							}
						}
					} else {
						cursorColumn = 1;
					}
					
					// Only update line numbers if we actually found a new position
					if (foundInDivs) {
						updateLineNumbers();
					}
					
					console.log("Cursor position updated:", {
						activeLineIndex,
						cursorLine,
						cursorColumn,
						totalLines: allDivs.length
					});
				}
			}
			
			// Only update line numbers if we actually found a new position
			if (foundInDivs) {
				updateLineNumbers();
			}
			
			console.log("Cursor position updated:", {
				activeLineIndex,
				cursorLine,
				cursorColumn,
				totalLines: allDivs.length
			});
		} catch (error) {
			console.error("Error updating cursor position:", error);
		}
	}

	// Helper function to get text before cursor in contenteditable
	function getTextBeforeCursor(node: Node, offset: number): string {
		if (!editorElement) return '';
		
		let text = '';
		
		// Handle text node
		if (node.nodeType === Node.TEXT_NODE) {
			text = (node.textContent || '').substring(0, offset);
		}
		
		// Go up the tree and collect text from nodes before this one
		let current = node;
		while (current !== editorElement) {
			const parent = current.parentNode;
			if (!parent) break;
			
			// Get all previous siblings
			let sibling = parent.firstChild;
			while (sibling && sibling !== current) {
				text = (sibling.textContent || '') + text;
				sibling = sibling.nextSibling;
			}
			
			// Move up the tree
			current = parent;
		}
		
		return text;
	}

	// Update the adjustTextareaHeight function to handle full page scrolling
	function adjustEditorHeight() {
		if (!editorElement) return;

		// Get the actual number of lines from DOM structure for more accuracy
		let numberOfLines; 

		// Count actual div elements (each div is a line)
		const divElements = editorElement.querySelectorAll('div');
		numberOfLines = divElements.length;
		
		// If no divs but there's content, ensure at least one line
		if (numberOfLines === 0 && editorElement.textContent && editorElement.textContent.trim().length > 0) {
			numberOfLines = 1;
		}
		
		// Ensure a minimum of 1 line
		numberOfLines = Math.max(1, numberOfLines);
		
		// Check if we need to add an extra line for cursor at end of document
		const selection = window.getSelection();
		if (selection && selection.rangeCount > 0) {
			const range = selection.getRangeAt(0);
			if (range.startContainer === editorElement && 
				range.startOffset === editorElement.childNodes.length) {
				// Cursor is after the last div, add an extra line
				numberOfLines++;
			}
		}
		
		console.log(`Line count from DOM: ${numberOfLines} divs`);
		
		// Calculate the height based on number of lines
		const contentHeight = numberOfLines * LINE_HEIGHT;
		
		// Calculate the minimum height based on MIN_LINES
		const minHeight = LINE_HEIGHT * MIN_LINES;

		// Set height to the larger of content height or minimum height, plus extra padding
		const newHeight = Math.max(contentHeight, minHeight) + 48; // Add extra padding
		
		console.log(`Adjusting editor height: lines=${numberOfLines}, newHeight=${newHeight}px`);

		// Apply the new height to editor without overflow
		editorElement.style.height = `${newHeight}px`;
		
		// Remove overflow scrolling from editor element - we want page scrolling, not editor scrolling
		editorElement.style.overflowY = 'visible';
		editorElement.style.maxHeight = 'none'; // Remove max height constraint

		// Update the line numbers container height
		const lineNumbersContainer = document.querySelector('.line-numbers') as HTMLElement;
		if (lineNumbersContainer) {
			lineNumbersContainer.style.height = `${newHeight}px`;
			lineNumbersContainer.style.overflowY = 'visible'; // Remove scrolling to match editor
		}

		// Update the editor wrapper - this is the container with the transparent backdrop
		const editorWrapper = document.querySelector('.editor-wrapper.current') as HTMLElement;
		if (editorWrapper) {
			editorWrapper.style.height = 'auto'; // Let it grow naturally with content
			editorWrapper.style.minHeight = `${newHeight}px`; // Ensure minimum height
		}

		// Update the editor content container
		const editorContentElement = document.querySelector('.editor-content') as HTMLElement;
		if (editorContentElement) {
			editorContentElement.style.minHeight = `${newHeight}px`;
		}
	}

	// Add this function to ensure cursor is visible
	function ensureCursorVisible() {
		if (!editorElement) return;
		
		const selection = window.getSelection();
		if (!selection || !selection.rangeCount) return;
		
		const range = selection.getRangeAt(0);
		const rect = range.getBoundingClientRect();
		
		// Check if cursor is visible in viewport
		const isVisible = (
			rect.top >= 0 &&
			rect.left >= 0 &&
			rect.bottom <= (window.innerHeight || document.documentElement.clientHeight) &&
			rect.right <= (window.innerWidth || document.documentElement.clientWidth)
		);
		
		if (!isVisible) {
			// Scroll to make cursor visible
			window.scrollTo({
				top: window.scrollY + rect.top - window.innerHeight/2,
				behavior: 'smooth'
			});
		}
	}

	// Update the onMount function for proper initialization
	onMount(async () => {
		// Check if we're in a browser environment first
		if (!browser) return;
		
		console.log("onMount: Initializing editor");
		
		// Load document data and profile image in parallel
		try {
			const [docResult, userResult] = await Promise.all([
				loadDocumentData(),
				loadUserProfile()
			]);
			
			console.log("onMount: Document loaded, content length:", editorContent?.length || 0);
			
			// Set navbar ready first
			navbarReady = true;
			
			// Then set document ready with a shorter delay
			setTimeout(() => {
				documentReady = true;
				
				// Force refresh line numbers and layout after everything is loaded
				setTimeout(() => {
					// Set initial cursor and focus the editor
					if (editorElement) {
						console.log("Editor element found in onMount:", editorElement);
						
						// Ensure content is set
						if (editorContent && (!editorElement.innerHTML || editorElement.innerHTML.trim() === '')) {
							console.log("Editor was empty, setting content from editorContent");
							safelySetEditorContent(editorContent);
						}
						
						// Force a complete refresh of line numbers based on actual content
						const divCount = editorElement.querySelectorAll('div').length;
						console.log(`Initial line count: ${divCount} divs`);
						
						// Update lines array based on actual content
						lines = editorContent.split('\n');
						if (lines.length === 0) {
							lines = [''];
						}
						
						// Position cursor at the beginning
						const selection = window.getSelection();
						const range = document.createRange();
						
						// Select the first text node or the editor element itself if empty
						if (editorElement.firstChild) {
							range.setStart(editorElement.firstChild, 0);
						} else {
							range.setStart(editorElement, 0);
						}
						range.collapse(true);
						
						selection?.removeAllRanges();
						selection?.addRange(range);
						
						// Set active line to first line
						activeLineIndex = 0;
						cursorLine = 1;
						cursorColumn = 1;
						
						// Focus the editor
						editorElement.focus();
						
						// Force update line numbers
						updateLineNumbers();
						
						// Add click event listener for cursor position updates
						editorElement.addEventListener('click', (event) => {
							// Prevent any default click behavior that might interfere
							event.preventDefault();
							
							if (!editorElement) return;
							
							// Get the clicked div
							const selection = window.getSelection();
							if (!selection || !selection.rangeCount) return;
							
							const range = selection.getRangeAt(0);
							const clickedNode = range.startContainer;
							
							// Find the div that was clicked
							let currentDiv: Node | null = clickedNode;
							while (currentDiv && currentDiv.parentElement !== editorElement) {
								currentDiv = currentDiv.parentElement;
							}
							
							if (currentDiv && currentDiv instanceof HTMLDivElement) {
								// Get all divs to find the index
								const allDivs = Array.from(editorElement.querySelectorAll('div'));
								const clickedIndex = allDivs.indexOf(currentDiv);
								
								if (clickedIndex !== -1) {
									// Update indices directly
									activeLineIndex = clickedIndex;
									cursorLine = clickedIndex + 1;
									
									// Calculate cursor column
									if (clickedNode.nodeType === Node.TEXT_NODE) {
										cursorColumn = range.startOffset + 1;
									} else {
										cursorColumn = 1;
									}
									
									// Update line numbers only once
									updateLineNumbers();
									console.log('Click handler updated position:', {
										activeLineIndex,
										cursorLine,
										cursorColumn
									});
								}
							}
						});
						
						// Add selection change listener to track cursor movement
						document.addEventListener('selectionchange', () => {
							if (document.activeElement === editorElement) {
								updateCursorPosition();
								updateLineNumbers();
							}
						});
						
						// Update UI
						updateCursorPosition();
						updateLineNumbers();
						adjustEditorHeight();
					}
					
					// Set up a MutationObserver to watch for content changes
					if (editorElement) {
						const observer = new MutationObserver((mutations) => {
							// Content might have changed, ensure line numbers are updated
							const content = getEditorContent();
							if (content !== editorContent) {
								console.log('MutationObserver detected content change');
								editorContent = content;
								updateLineNumbers();
								updateCursorPosition();
								adjustEditorHeight();
							}
						});
						
						// Watch for text and child node changes
						observer.observe(editorElement, {
							childList: true,
							characterData: true,
							subtree: true
						});
					}
				}, 100);
			}, 150);
			
		} catch (e) {
			console.error('Error during initialization:', e);
			error = true;
		} finally {
			loading = false;
		}
	});

	// Separate cleanup function for event listeners
	onDestroy(() => {
		if (!browser) return;
		
		// Remove scroll and resize event listeners
		window.removeEventListener('scroll', () => {});
		window.removeEventListener('resize', () => {});
		
		// Clean up auto-save if it exists
		if (autoSaveCleanup) {
			autoSaveCleanup();
		}
	});

	// Function to load document data
	async function loadDocumentData() {
		try {
			documentData = await get_document(parseInt(documentId));
			console.log("Document data loaded:", documentData);

			if (documentData) {
				// Get the HTML content from the document
				let htmlContent = documentData.content || '';
				console.log("Raw HTML content:", htmlContent);
				
				// If the content is empty, create a default empty div
				if (htmlContent.trim() === '') {
					htmlContent = '<div><br></div>';
				}
				
				// If content doesn't have div tags, convert it to HTML format
				if (!htmlContent.includes('<div')) {
					// Convert plain text lines to HTML with divs
					const lines = htmlContent.split('\n');
					htmlContent = lines.map((line: string) => {
						// If line is empty, add a <br> tag to preserve it
						return line.trim() === '' ? '<div><br></div>' : `<div>${line}</div>`;
					}).join('');
					console.log("Converted to HTML:", htmlContent);
				}
				
				// If content doesn't have div tags, convert it to HTML format
				if (!htmlContent.includes('<div')) {
					// Convert plain text lines to HTML with divs
					const lines = htmlContent.split('\n');
					htmlContent = lines.map((line: string) => {
						// If line is empty, add a <br> tag to preserve it
						return line.trim() === '' ? '<div><br></div>' : `<div>${line}</div>`;
					}).join('');
				}
				
				// Store the HTML content
				editorContent = htmlContent;
				console.log("Editor content set to:", editorContent);
				
				// Set the content in the editor
				if (editorElement) {
					// Directly set innerHTML to render the HTML structure
					console.log("Setting innerHTML on editorElement:", htmlContent);
					editorElement.innerHTML = htmlContent;
				} else {
					console.warn("Editor element not available yet for setting content");
					// Set up a retry mechanism for when the editor element becomes available
					const maxRetries = 5;
					let retryCount = 0;
					
					const retryInterval = setInterval(() => {
						if (editorElement) {
							console.log("Editor element now available, setting content");
							editorElement.innerHTML = htmlContent;
							clearInterval(retryInterval);
							
							// Update UI after setting content
							updateLineNumbers();
							updateCursorPosition();
							adjustEditorHeight();
						} else if (retryCount >= maxRetries) {
							console.error("Failed to set editor content after max retries");
							clearInterval(retryInterval);
						}
						retryCount++;
					}, 200);
				}

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
					if (documentData && editorElement) {
						// Save the inner HTML of the editor element
						documentData.content = editorElement.innerHTML;
						console.log(editorElement.innerHTML)
						update_document(documentData);
					}
				});

				// Initialize line numbers when document loads
				setTimeout(() => {
					updateLineNumbers();
					updateCursorPosition();
					adjustEditorHeight();
				}, 50);
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
	
	// Improved handleInput function to work in coordination with handleKeyDown
	function handleInput(event: Event) {
		// In NORMAL mode, prevent any changes to content
		if (editorMode === 'NORMAL') {
			// Restore the original content to prevent changes
			setEditorContent(editorContent);
			event.preventDefault();
			return;
		}
		
		// Only allow editing in INSERT mode
		if (editorMode === 'INSERT') {
			// Get the actual content directly from the element
			const newContent = getEditorContent();
			
			// Only process if content actually changed
			if (newContent !== editorContent) {
				// Log for debugging the content change
				console.log('Content changed via input event:', {
					oldLines: editorContent.split('\n').length, 
					newLines: newContent.split('\n').length
				});
				
				// Update our content tracking
				editorContent = newContent;
				
				// Always update line numbers first based on the new content
				updateLineNumbers();
				
				// Check if wrapping is needed
				const contentLines = editorContent.split('\n');
				let needsWrapping = false;
				
				for (const line of contentLines) {
					if (line.length > MAX_COLUMN_WIDTH) {
						needsWrapping = true;
						break;
					}
				}
				
				// Apply wrapping if needed
				if (needsWrapping) {
					const selection = window.getSelection();
					const range = selection?.getRangeAt(0);
					const cursorOffset = range ? getTextOffset(range.startContainer, range.startOffset) : 0;
					
					// Apply wrapping
					const wrappedContent = autoWrapLine(editorContent);
					
					// Only update if wrapping actually changed something
					if (wrappedContent !== editorContent) {
						editorContent = wrappedContent;
						setEditorContent(wrappedContent);
						
						// Attempt to restore cursor position
						setCursorPositionByOffset(Math.min(cursorOffset, editorContent.length));
					}
				} else {
					// If no wrapping needed, still update cursor position and height
					updateCursorPosition();
					adjustEditorHeight();
				}
			}
		}
	}
	
	// Improved getEditorContent function to handle newlines consistently
	function getEditorContent(): string {
		if (!editorElement) return '';
		
		let content = '';

		// For contenteditable, use div structure to preserve empty lines accurately
		const divElements = editorElement.querySelectorAll('div');
		
		if (divElements.length > 0) {
			// Collect text from each div, preserving empty lines
			const lines = Array.from(divElements).map(div => {
				// Get text content, replacing zero-width spaces with nothing
				let text = div.textContent || '';
				text = text.replace(/\u200B/g, ''); // Remove zero-width spaces
				return text;
			});
			
			// Join with newlines to form content
			content = lines.join('\n');
			
			// Check if we need an extra newline at the end
			const selection = window.getSelection();
			if (selection && selection.rangeCount > 0) {
				const range = selection.getRangeAt(0);
				if (range.startContainer === editorElement && 
					range.startOffset === editorElement.childNodes.length) {
					// Cursor is after the last div, add an extra newline
					content += '\n';
				}
			}
		} else {
			// No divs, use innerText and normalize
			content = editorElement.innerText || '';
		}
		
		// Check if we need to normalize line breaks (some browsers use different conventions)
		if (content.includes('\r\n')) {
			// Convert Windows-style CRLF to just LF
			content = content.replace(/\r\n/g, '\n');
		}
		
		// Log for debugging
		console.log(`getEditorContent: ${content.split('\n').length} lines (${content.split('\n').filter(l => l === '').length} empty)`);
	
		return content;
	}
	
	// Improved setEditorContent function to handle line counting correctly
	function setEditorContent(content: string) {
		if (!editorElement) return;
		
		// Normalize empty content
		if (content.trim() === '' || content === '\n') {
			content = '';
		}
		
		// Use our safe helper method
		safelySetEditorContent(content);
		
		// Force a complete update of line numbers and UI
		setTimeout(() => {
			// Log for debugging
			console.log('Content set:', {
				contentLines: content.split('\n').length,
				charCount: content.length,
				isEmpty: content === ''
			});
			
			// Update line numbers and UI
			updateLineNumbers();
			updateCursorPosition();
			adjustEditorHeight();
		}, 0);
	}
	
	// Helper function to get text offset in contenteditable div
	function getTextOffset(node: Node, offset: number): number {
		if (!editorElement) return 0;
		
		const treeWalker = document.createTreeWalker(
			editorElement,
			NodeFilter.SHOW_TEXT,
			null
		);
		
		let currentOffset = 0;
		let currentNode = treeWalker.nextNode();
		
		while (currentNode) {
			if (currentNode === node) {
				return currentOffset + offset;
			}
			
			currentOffset += (currentNode.textContent || '').length;
			currentNode = treeWalker.nextNode();
		}
		
		return 0;
	}

	// Helper function to set cursor position by character offset
	function setCursorPositionByOffset(offset: number) {
		if (!editorElement) return;
		
		const treeWalker = document.createTreeWalker(
			editorElement,
			NodeFilter.SHOW_TEXT,
			null
		);
		
		let currentOffset = 0;
		let currentNode = treeWalker.nextNode();
		
		while (currentNode) {
			const nodeLength = (currentNode.textContent || '').length;
			
			if (currentOffset + nodeLength >= offset) {
				const range = document.createRange();
				const sel = window.getSelection();
				
				range.setStart(currentNode, offset - currentOffset);
				range.collapse(true);
				
				sel?.removeAllRanges();
				sel?.addRange(range);
				return;
			}
			
			currentOffset += nodeLength;
			currentNode = treeWalker.nextNode();
		}
	}

	// Add these formatting functions after the performRedo function
	function applyBoldFormatting() {
		if (document.queryCommandSupported('bold')) {
			document.execCommand('bold', false);
			showCommandError('Bold formatting applied');
		}
	}

	function applyItalicFormatting() {
		if (document.queryCommandSupported('italic')) {
			document.execCommand('italic', false);
			showCommandError('Italic formatting applied');
		}
	}

	function applyIndentation() {
		document.execCommand('indent', false);
		showCommandError('Indentation applied');
	}

	function applyTextColor(color: string) {
		if (document.queryCommandSupported('foreColor')) {
			document.execCommand('foreColor', false, color);
			showCommandError('Text color changed');
		}
	}

	// Add this helper function to implement setSelectionRange for contenteditable divs
	function setRange(element: HTMLElement, start: number, end: number) {
		if (!element) return;
		
		const selection = window.getSelection();
		const range = document.createRange();
		let charCount = 0;
		let foundStart = false;
		let foundEnd = false;
		
		function traverse(node: Node) {
			if (foundEnd) return;
			
			if (node.nodeType === Node.TEXT_NODE) {
				const nextCharCount = charCount + node.textContent!.length;
				
				// Set start position
				if (!foundStart && start >= charCount && start <= nextCharCount) {
					range.setStart(node, start - charCount);
					foundStart = true;
				}
				
				// Set end position
				if (foundStart && !foundEnd && end >= charCount && end <= nextCharCount) {
					range.setEnd(node, end - charCount);
					foundEnd = true;
				}
				
				charCount = nextCharCount;
			} else {
				const childNodes = node.childNodes;
				for (let i = 0; i < childNodes.length; i++) {
					traverse(childNodes[i]);
				}
			}
		}
		
		traverse(element);
		
		if (selection) {
			selection.removeAllRanges();
			selection.addRange(range);
		}
	}

	// Add a handlePaste function for contenteditable
	function handlePaste(event: ClipboardEvent) {
		event.preventDefault();
		
		// Get the clipboard text
		const clipboardText = event.clipboardData?.getData('text/plain') || '';
		if (!clipboardText) return;
		
		// Get the current selection
		const selection = window.getSelection();
		if (!selection || !selection.rangeCount) return;
		
		const range = selection.getRangeAt(0);
		const start = getTextOffset(range.startContainer, range.startOffset);
		const end = getTextOffset(range.endContainer, range.endOffset);
		
		// Insert the clipboard text at the current cursor position
		const beforePaste = editorContent.substring(0, start);
		const afterPaste = editorContent.substring(end);
		const newContent = beforePaste + clipboardText + afterPaste;
		
		editorContent = newContent;
		
		// Update the editor content using our safe method
		safelySetEditorContent(newContent);
		
		// Set cursor position after the pasted text
		if (editorElement) {
			setRange(editorElement, start + clipboardText.length, start + clipboardText.length);
		}
		
		// Update line numbers and other UI elements
		lines = editorContent.split('\n');
		updateCursorPosition();
		adjustEditorHeight();
	}

	// Add export to PDF function
	function exportToPDF() {
		if (!browser || !documentData) return;
		
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
			showCommandError('PDF exported successfully');
		} catch (error) {
			console.error('Error exporting PDF:', error);
			showCommandError('Failed to export PDF');
		}
	}

	// Redesigned updateLineNumbers function for better empty line handling
	function updateLineNumbers() {
		if (!editorElement) return;
		
		console.log("updateLineNumbers called");
		console.log("Current activeLineIndex:", activeLineIndex);
		console.log("Current cursorLine:", cursorLine);
		
		// Determine line count based on editor content
		let lineCount = 1; // Start with at least one line
		
		// First check if we have any div elements (paragraphs)
		const divElements = editorElement.querySelectorAll('div');
		const divCount = divElements.length;
		console.log(`Found ${divCount} divs for line numbering`);
		
		// Debug current selection state
		const selection = window.getSelection();
		if (selection && selection.rangeCount > 0) {
			const range = selection.getRangeAt(0);
			console.log("Current selection:", {
				startContainer: range.startContainer.nodeType === Node.TEXT_NODE ? 'TEXT_NODE' : 'ELEMENT_NODE',
				startOffset: range.startOffset,
				endContainer: range.endContainer.nodeType === Node.TEXT_NODE ? 'TEXT_NODE' : 'ELEMENT_NODE',
				endOffset: range.endOffset,
				collapsed: range.collapsed
			});
		}
		
		// If we have divs, count them (each is a paragraph/line)
		if (divCount > 0) {
			lineCount = divCount;
			
			// Debug each div's content
			divElements.forEach((div, index) => {
				console.log(`Div ${index}: "${div.textContent}" (length: ${div.textContent?.length})`);
			});
			
			// Double-check if there should be an extra line at the end (if cursor is after last div)
			if (selection && selection.rangeCount > 0) {
				const range = selection.getRangeAt(0);
				if (range.startContainer === editorElement && 
					range.startOffset === editorElement.childNodes.length) {
					// Cursor is after the last div, may need an extra line
					lineCount++;
					console.log("Added extra line for cursor at end");
				}
			}
		} else {
			// No divs, check for other line separators
			const brElements = editorElement.querySelectorAll('br');
			console.log(`No divs found, but found ${brElements.length} <br> elements`);
			
			if (brElements.length > 0) {
				// Each br tag creates a new line
				lineCount = brElements.length + 1;
			} else {
				// If there's any content at all, ensure at least one line
				const hasContent = editorElement.textContent && editorElement.textContent.trim().length > 0;
				lineCount = hasContent ? 1 : 1; // Always at least one line
				console.log(`No divs or <br>, content present: ${hasContent}`);
				
				// Also check for newlines in the text content
				const newlineCount = (editorElement.textContent?.match(/\n/g) || []).length;
				if (newlineCount > 0) {
					lineCount = Math.max(lineCount, newlineCount + 1);
					console.log(`Found ${newlineCount} newlines in text content`);
				}
			}
		}

		// Ensure at least one line
		lineCount = Math.max(1, lineCount);
		console.log(`Final line count: ${lineCount}`);
		
		// Extract text from lines for contenteditable
		const newLines = [];
		const divs = editorElement.querySelectorAll('div');
		
		if (divs.length > 0) {
			// Get text from each div
			divs.forEach((div, index) => {
				const lineText = div.textContent || '';
				newLines.push(lineText);
				console.log(`Line ${index}: "${lineText}"`);
			});
			
			// Add an empty line if the cursor suggests there should be one
			if (selection && selection.rangeCount > 0) {
				const range = selection.getRangeAt(0);
				if (range.startContainer === editorElement && 
					range.startOffset === editorElement.childNodes.length) {
					newLines.push('');
					console.log("Added empty line at end due to cursor position");
				}
			}
		} else {
			// Split text by <br> tags or newlines
			const textContent = editorElement.textContent || '';
			const htmlContent = editorElement.innerHTML;
			console.log("Editor HTML content:", htmlContent);
			
			// Check if we have <br> tags
			if (htmlContent.includes('<br')) {
				// Replace <br> tags with newlines for splitting
				const withNewlines = htmlContent.replace(/<br\s*\/?>/gi, '\n');
				// Remove other HTML tags
				const textOnly = withNewlines.replace(/<[^>]*>/g, '');
				// Split by newlines
				newLines.push(...textOnly.split('\n'));
				console.log("Split content by <br> tags:", newLines);
			} else {
				// Just split by actual newlines
				newLines.push(...textContent.split('\n'));
				console.log("Split content by newlines:", newLines);
			}
			
			// If we still don't have any lines, add one empty line
			if (newLines.length === 0) {
				newLines.push('');
				console.log("Added default empty line");
			}
		}
		
		// Update lines array
		lines = newLines.length > 0 ? newLines : [''];
		console.log(`Lines array updated with ${lines.length} entries:`, lines);
		
		// Get line numbers container - be more specific with selector
		const lineNumbersContainer = document.querySelector('.editor-content .line-numbers');
		console.log("Line numbers container:", lineNumbersContainer);
		if (!lineNumbersContainer) {
			console.error("Could not find line numbers container");
			return;
		}
		
		// Clear existing line numbers
		lineNumbersContainer.innerHTML = '';
		console.log("Cleared existing line numbers");
		
		// Create line number elements - ensure activeLineIndex is in bounds
		const previousActiveLineIndex = activeLineIndex;
		activeLineIndex = Math.max(0, Math.min(activeLineIndex, lineCount - 1));
		console.log(`Active line index adjusted from ${previousActiveLineIndex} to ${activeLineIndex}`);
		
		// Create all line number elements
		console.log(`Creating ${lineCount} line number elements`);
		for (let i = 0; i < lineCount; i++) {
			const lineNumber = document.createElement('div');
			lineNumber.className = 'line-number';
			
			// Add active class with a clearer, more pronounced style
			if (i === activeLineIndex) {
				lineNumber.classList.add('active');
				console.log(`Highlighting line number ${i + 1} as active`);
			}
			
			lineNumber.textContent = (i + 1).toString();
			lineNumbersContainer.appendChild(lineNumber);
		}
		
		console.log("Line numbers updated successfully");
		console.log("Final state:", {
			lineCount,
			activeLineIndex,
			cursorLine,
			cursorColumn,
			totalLines: lines.length
		});
	}

	// Add performUndo function
	function performUndo() {
		if (document.queryCommandSupported('undo')) {
			document.execCommand('undo', false);
			showCommandError('Undo operation performed');
		}
	}
	
	// Add performRedo function 
	function performRedo() {
		if (document.queryCommandSupported('redo')) {
			document.execCommand('redo', false);
			showCommandError('Redo operation performed');
		}
	}

	// Add navigation functions
	function moveToStartOfLine() {
		if (!editorElement) return;
		
		// For contenteditable divs, use DOM approach
		const selection = window.getSelection();
		if (!selection || !selection.rangeCount) return;
		
		const range = selection.getRangeAt(0);
		
		// Find the current div element
		let currentDiv: Node | null = range.startContainer;
		
		// If the startContainer is a text node, get its parent
		if (currentDiv.nodeType === Node.TEXT_NODE) {
			currentDiv = currentDiv.parentNode;
		}
		
		// Traverse up the DOM until we find a div that's a direct child of the editor
		while (currentDiv && currentDiv !== editorElement && currentDiv.parentNode !== editorElement) {
			currentDiv = currentDiv.parentNode;
		}
		
		// Make sure we found a div
		if (currentDiv && currentDiv !== editorElement) {
			// Create a new range at the start of the div
			const newRange = document.createRange();
			
			// If the div has content, position at the start of the first text node
			if (currentDiv.firstChild && currentDiv.firstChild.nodeType === Node.TEXT_NODE) {
				newRange.setStart(currentDiv.firstChild, 0);
			} else {
				// If the div is empty, position at the start of the div
				newRange.setStart(currentDiv, 0);
			}
			newRange.collapse(true);
			
			// Apply the range
			selection.removeAllRanges();
			selection.addRange(newRange);
			
			// Update cursor position
			cursorColumn = 1;
			updateCursorPosition();
		}
	}
	
	function moveToEndOfLine() {
		if (!editorElement) return;
		
		// For contenteditable divs, use DOM approach
		const selection = window.getSelection();
		if (!selection || !selection.rangeCount) return;
		
		const range = selection.getRangeAt(0);
		
		// Find the current div element
		let currentDiv: Node | null = range.startContainer;
		
		// If the startContainer is a text node, get its parent
		if (currentDiv.nodeType === Node.TEXT_NODE) {
			currentDiv = currentDiv.parentNode;
		}
		
		// Traverse up the DOM until we find a div that's a direct child of the editor
		while (currentDiv && currentDiv !== editorElement && currentDiv.parentNode !== editorElement) {
			currentDiv = currentDiv.parentNode;
		}
		
		// Make sure we found a div
		if (currentDiv && currentDiv !== editorElement) {
			// Create a new range at the end of the div
			const newRange = document.createRange();
			
			// If the div has content, position at the end of the last text node
			if (currentDiv.lastChild && currentDiv.lastChild.nodeType === Node.TEXT_NODE) {
				const textNode = currentDiv.lastChild;
				const length = textNode.textContent?.length || 0;
				newRange.setStart(textNode, length);
			} else {
				// If the div is empty, position at the start of the div
				newRange.setStart(currentDiv, 0);
			}
			newRange.collapse(true);
			
			// Apply the range
			selection.removeAllRanges();
			selection.addRange(newRange);
			
			// Update cursor position
			updateCursorPosition();
		}
	}
	
	// Function to move to the end of the document (G command)
	function moveToEndOfDocument() {
		if (!editorElement) return;
		
		console.log("Executing 'G' command - moving to last line");
		
		// For contenteditable divs, use DOM approach
		const allDivs = Array.from(editorElement.querySelectorAll('div'));
		
		if (allDivs.length > 0) {
			// Get the last div
			const lastDiv = allDivs[allDivs.length - 1];
			
			// Create a range at the end of the last div
			const range = document.createRange();
			
			// Check if the last div has any content
			if (lastDiv.lastChild && lastDiv.lastChild.nodeType === Node.TEXT_NODE) {
				// If it has text content, put cursor at the end of the text
				const textNode = lastDiv.lastChild;
				const length = textNode.textContent?.length || 0;
				range.setStart(textNode, length);
			} else {
				// If empty, just set cursor at the beginning of the div
				range.setStart(lastDiv, 0);
			}
			range.collapse(true);
			
			// Apply the range to move the cursor
			const selection = window.getSelection();
			if (selection) {
				editorElement.focus();
				selection.removeAllRanges();
				selection.addRange(range);
			}
			
			// Update indices
			activeLineIndex = allDivs.length - 1;
			cursorLine = allDivs.length;
			
			// Set cursor column to the end of the line
			const lineText = lastDiv.textContent || '';
			cursorColumn = lineText.length + 1;
		} else {
			// No divs, just go to the end of the content
			const length = editorContent.length;
			setRange(editorElement, length, length);
		}
		
		// Update UI
		updateCursorPosition();
		updateLineNumbers();
		ensureCursorVisible();
	}

	// Add user account functions
	function goToAccount() {
		goto('/account');
	}
	
	function handleLogout() {
		logout().then(() => {
			goto('/login');
		});
	}
	
	// Add command execution function
	function executeCommand(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			event.preventDefault();
			
			if (commandPrefix === ':') {
				const success = handleColonCommand(commandInput);
				if (success) {
					exitCommandMode();
				}
			} else if ((commandPrefix === '/' || commandPrefix === '?') && searchResults.length > 0) {
				navigateToSearchResult();
				exitCommandMode();
			}
		} else if (event.key === 'Escape') {
			exitCommandMode();
		}
	}
	
	// Add helper function for calculating max column width
	function calculateMaxColumnWidth(): number {
		if (!editorElement) return MAX_COLUMN_WIDTH;
		
		// Get font metrics
		const style = window.getComputedStyle(editorElement);
		const font = style.font;
		
		// Create a temporary span to measure character width
		const span = document.createElement('span');
		span.style.font = font;
		span.style.position = 'absolute';
		span.style.visibility = 'hidden';
		span.textContent = 'X'.repeat(100); // Use a representative character
		
		document.body.appendChild(span);
		const charWidth = span.getBoundingClientRect().width / 100;
		document.body.removeChild(span);
		
		// Calculate how many characters fit in the editor width with some margin
		const editorWidth = editorElement.clientWidth - 40; // 20px padding on each side
		const maxChars = Math.floor(editorWidth / charWidth);
		
		return Math.max(60, Math.min(maxChars, 120)); // Keep between 60-120 chars
	}

	// Helper function to get node offset within a specific parent
	function getNodeOffsetWithinParent(node: Node, parentDiv: Node, offset: number): number {
		if (!node || !parentDiv) return offset;
		
		// If node is a text node and it's directly in the parent div
		if (node.nodeType === Node.TEXT_NODE && node.parentNode === parentDiv) {
			return offset;
		}
		
		// Calculate text length before this node in the parent div
		let textBeforeNode = 0;
		
		// Function to traverse the parent's contents
		function traverseParent(currentNode: Node) {
			if (currentNode === node) {
				// Found our node, stop here
				return true;
			}
			
			if (currentNode.nodeType === Node.TEXT_NODE) {
				// Add text content length
				textBeforeNode += (currentNode.textContent || '').length;
			} else if (currentNode.nodeType === Node.ELEMENT_NODE) {
				// Traverse child nodes
				for (let i = 0; i < currentNode.childNodes.length; i++) {
					if (traverseParent(currentNode.childNodes[i])) {
						return true;
					}
				}
			}
			
			return false;
		}
		
		// Start traversal on parent's children
		for (let i = 0; i < parentDiv.childNodes.length; i++) {
			if (traverseParent(parentDiv.childNodes[i])) {
				break;
			}
		}
		
		// Return the text before + offset
		return textBeforeNode + offset;
	}

	// Helper function to get current selection offset
	function getSelectionOffset(): number {
		const selection = window.getSelection();
		if (!selection || !selection.rangeCount) return 0;
		
		const range = selection.getRangeAt(0);
		return getTextOffset(range.startContainer, range.startOffset);
	}

	// Helper function to safely set editor content while preserving div structure
	function safelySetEditorContent(content: string) {
		if (!editorElement) return;
		
		// Important check: verify we're not accidentally removing lines
		const currentDivCount = editorElement.querySelectorAll('div').length;
		const newLineCount = content.split('\n').length;
		
		// If we're reducing the number of lines and in normal mode with a buffer containing just 'g' or 'd',
		// prevent accidental content replacement
		if (editorMode === 'NORMAL' && 
		   (normalModeBuffer === 'g' || normalModeBuffer === 'd') && 
		   newLineCount < currentDivCount) {
			console.warn(`Prevented content change that would remove lines during incomplete ${normalModeBuffer} command`);
			return;
		}
		
		// Always use the div structure approach for consistency
		// Clear the editor
		editorElement.innerHTML = '';
		
		// Split content into lines
		const lines = content.split('\n');
		
		// Handle the case with no lines - ensure at least one empty div
		if (lines.length === 0) {
			const div = document.createElement('div');
			editorElement.appendChild(div);
			return;
		}
		
		// Create a div for each line, properly handling empty lines
		lines.forEach((line, index) => {
			const div = document.createElement('div');
			// Even completely empty lines need a div
			div.textContent = line; // Setting textContent works for empty lines too
			if(editorElement) {
				editorElement.appendChild(div);	
			}
			
		});
		
		// Debug
		console.log(`Set editor content: ${lines.length} lines (${lines.filter(l => l === '').length} empty)`);
		
		// Force browser to recognize empty divs 
		// This prevents browser optimization from collapsing empty divs
		const emptyDivs = Array.from(editorElement.querySelectorAll('div')).filter(div => div.textContent === '');
		emptyDivs.forEach(div => {
			// Add a zero-width space character to force the browser to keep the div
			if (!div.firstChild) {
				div.appendChild(document.createTextNode('\u200B')); // Zero-width space
			}
		});
	}

	// Function to move to the start of the document (gg command)
	function moveToStartOfDocument() {
		if (!editorElement) return;
		
		console.log("Executing 'gg' command - moving to first line");
		
		// For contenteditable divs, use DOM approach
		const allDivs = Array.from(editorElement.querySelectorAll('div'));
		
		if (allDivs.length > 0) {
			// Get the first div
			const firstDiv = allDivs[0];
			
			// Create a range at the start of the first div
			const range = document.createRange();
			range.setStart(firstDiv, 0);
			range.collapse(true);
			
			// Apply the range to move the cursor
			const selection = window.getSelection();
			if (selection) {
				editorElement.focus();
				selection.removeAllRanges();
				selection.addRange(range);
			}
			
			// Update indices
			activeLineIndex = 0;
			cursorLine = 1;
			cursorColumn = 1;
		} else {
			// No divs, just go to the start of the content
			setRange(editorElement, 0, 0);
		}
	
		// Update UI
		updateCursorPosition();
		updateLineNumbers();
		ensureCursorVisible();
	}

	// Add toast styles near other style definitions
	const toastSuccess = {
		theme: {
			'--toastBackground': '#48BB78',
			'--toastBarBackground': '#2F855A'
		}
	}

	const toastError = {
		theme: {
			'--toastBackground': '#F56565',
			'--toastBarBackground': '#C53030'
		}
	}
</script>

<svelte:head>
	<title>{documentData ? documentData.name : 'Document'} | Vynn</title>
</svelte:head>

{#each toasts as toast, i}
    <Toast 
        message={toast.message} 
        type={toast.type} 
        onClose={() => removeToast(i)} 
    />
{/each}

<div class="editor-page">
	<div class="background-image" style="background-image: url({backgroundImage})"></div>

	<!-- Minimal Navbar with fade-in animation -->
	<div class="navbar-container" class:fade-in-first={navbarReady}>
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
						src={userProfileImage} 
						alt="Profile" 
						class="rounded-circle profile-img"
						style="width: 40px; height: 40px; border: 2px solid var(--color-primary); margin-right: 10px; object-fit: cover;"
						on:error={() => userProfileImage = profileDefault}
					/>
				</button>
				<ul class="dropdown-menu dropdown-menu-end dropdown-menu-dark profile-dropdown">
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
		<div class="document-switcher fade-in-second">
			{#if projectDocuments.length > 0}
				{#each projectDocuments as doc, index}
					<button
						class="doc-button"
						class:active={doc.id.toString() === documentId}
						on:click={() => switchDocument(doc.id)}
						disabled={doc.id.toString() === documentId}
						aria-label="Switch to document {index + 1}: {doc.name || 'Untitled'}"
					>
						{index + 1}
					</button>
				{/each}
			{:else}
				<button
					class="doc-button active"
					disabled
					aria-label="Document 1"
				>
					1
				</button>
	{/if}
		</div>
	{/if}

	<!-- Editor Container with animation -->
	<div class="editor-container" class:fade-in-third={documentReady}>
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
						<div class="editor-contenteditable">{previousDocumentContent}</div>
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
						<!-- Line numbers now managed through JS for better synchronization -->
					</div>
					<div 
						bind:this={editorElement}
						class="editor-contenteditable" 
						contenteditable="true"
						on:keydown={handleKeyDown}
						on:input={handleInput}
						on:paste={handlePaste}
						spellcheck="false"
						role="textbox"
						aria-multiline="true"
						tabindex="0"
					></div>
				</div>
			</div>
		{/if}
	</div>

	<!-- Fixed Status Bar - moved outside the editor wrapper -->
	<div class="status-bar" class:fade-in-fourth={documentReady}>
		<div class="mode-indicator">
			<span class="mode {editorMode ? editorMode.toLowerCase() : 'normal'}">{editorMode || 'NORMAL'}</span>
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
			<button class="commands-toggle" on:click={() => showCommands = !showCommands} title="Toggle Commands Reference" aria-label="Toggle commands reference">
				<i class="bi bi-info-circle"></i>
			</button>
			<span>Line: {cursorLine}, Col: {cursorColumn}</span>
		</div>
	</div>

	<!-- Add commands cheat sheet overlay -->
	<div class="commands-overlay" class:show-commands={showCommands}>
		<div class="commands-header">
			<h5>Vim Command Reference</h5>
			<button class="commands-close" on:click={() => showCommands = false} aria-label="Close commands reference">×</button>
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
				<h6>Commands</h6>
				<ul>
					<li><span class="key">:q</span> Quit document</li>
					<li><span class="key">:w</span> Save document</li>
					<li><span class="key">:wq</span> Save and quit</li>
					<li><span class="key">:export</span> Export to PDF</li>
				</ul>
			</div>
			
			<div class="commands-section">
				<h6>Navigation</h6>
				<ul>
					<li><span class="key">h</span> Move left</li>
					<li><span class="key">j</span> Move down</li>
					<li><span class="key">k</span> Move up</li>
					<li><span class="key">l</span> Move right</li>
					<li><span class="key">0</span> Start of line</li>
					<li><span class="key">$</span> End of line</li>
					<li><span class="key">gg</span> Start of document</li>
					<li><span class="key">G</span> End of document</li>
				</ul>
			</div>
			
			<div class="commands-section">
				<h6>Editing</h6>
				<ul>
					<li><span class="key">x</span> Delete selected</li>
					<li><span class="key">dd</span> Delete line</li>
					<li><span class="key">yy</span> Copy line</li>
					<li><span class="key">p</span> Paste from yank</li>
				</ul>
			</div>
			
			<div class="commands-section">
				<h6>Search & Replace</h6>
				<ul>
					<li><span class="key">/</span> Search relative forward</li>
					<li><span class="key">?</span> Search relative backward</li>
					<li><span class="key">n</span> Next match</li>
					<li><span class="key">m</span> Previous match</li>
					<li><span class="key">:%s/old/new/g</span> Replace all</li>
				</ul>
			</div>

			<div class="commands-section">
				<h6>Editor Shortcuts</h6>
				<ul>
					<li><span class="key">Ctrl+/</span> Toggle this cheat sheet</li>
					<li><span class="key">Ctrl+1-9</span> Switch to document number</li>
				</ul>
			</div>
		</div>
	</div>
</div>