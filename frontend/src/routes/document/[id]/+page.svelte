<script lang="ts">
	import { onMount, onDestroy, afterUpdate } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { jsPDF } from 'jspdf';
	import { browser } from '$app/environment';
	import Toast from '$lib/components/Toast.svelte';
	
	import { get_document, update_document, get_project_from_document, setup_auto_save } from '$lib/ts/document';
	import { logout, get_current_user, get_profile_image_url } from '$lib/ts/user';
	import { get_project_documents } from '$lib/ts/project';
	import { keybindings, keybindingMap, type CommandFunctions, type KeyboardInput } from '$lib/ts/keybindings';

	import logo from '$lib/assets/logo.png';
	import backgroundImage from '$lib/assets/editor-background.jpg';
	import profileDefault from '$lib/assets/profile-image.png';

	import '$lib/assets/style/document.css';

	import ChatAssistant from '$lib/components/ChatAssistant.svelte';

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
	let documentReady = false; // to track when the document is ready to display
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
	let searchDirection: 'forward' | 'backward' = 'forward';
	let lastSearchQuery = '';
	let lastSearchDirection: 'forward' | 'backward' = 'forward';

	// Add variables for error messages
	let commandError = '';
	let commandErrorTimeout: ReturnType<typeof setTimeout> | null = null;

	// Add clipboard state
	let clipboardText = '';
	let normalModeBuffer = '';
	let normalModeBufferTimeout: ReturnType<typeof setTimeout> | null = null;

	// Add state for commands overlay
	let showCommands = false;
	let showColorPicker = false;

	// User profile data
	let userId: number | null = null;
	let userProfileImage = profileDefault;

	let showCommandSheet = false;
	let chatAssistantComponent: ChatAssistant;
	let isChatOpen = false; // Declare state variable for chat visibility
	let chatInputElementRef: HTMLInputElement | null = null; // Add ref for chat input

	// Add a function to prevent default browser behavior for certain key combinations
	function preventBrowserDefaults(event: KeyboardEvent) {
		// Prevent OS shortcuts by capturing all Ctrl/Cmd combinations
		if (event.ctrlKey || event.metaKey) {
			// Allow only specific browser shortcuts we want to keep
			const allowedKeys = ['c', 'v', 'a', 'z', 'y'];
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

				// Start animation without modifying any dimensions
				isAnimating = true;

				// Prevent scrolling during animation
				document.documentElement.style.overflow = 'hidden';

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
						const htmlWithDivs = lines
							.map((line) => (line.trim() === '' ? '<div><br></div>' : `<div>${line}</div>`))
							.join('');

						// Set the properly formatted HTML
						editorElement.innerHTML = htmlWithDivs;
					} else if (divCount === 0) {
						// Empty document case - add at least one empty div
						editorElement.innerHTML = '<div><br></div>';
						console.log('Empty document: adding empty div');
					}

					// First update immediately
					updateLineNumbers();

					// Then update after a short delay to ensure the DOM has stabilized
					setTimeout(() => {
						console.log('Refreshing line numbers after document switch');
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
							console.log('Refreshing line numbers after document switch');
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
					// Reset animation variables
					isAnimating = false;
					slideDirection = '';
					previousDocumentContent = '';
					previousDocumentLines = [];
					
					// Reset wrapper sizing constraints from animation
					const currentWrapper = document.querySelector('.editor-wrapper.current') as HTMLElement;
					if (currentWrapper) {
						// Clear minHeight to let it resize naturally
						currentWrapper.style.minHeight = '';
						currentWrapper.style.position = 'relative';
					}
					
					// Restore scrolling
					document.documentElement.style.overflow = '';
					
					// Gently adjust editor height after animation completes
					setTimeout(() => {
						// Smoothly adjust the editor height to match content
						if (editorElement) {
							adjustEditorHeight();
						}
					}, 50);
				}, 500); // Match this with the CSS animation duration (0.5s)

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
				console.log(`Fetching documents for project ID: ${projectInfo.project_id}`); // Add log
				// Fetch the actual documents for the project
				const documents = await get_project_documents(projectInfo.project_id);
				console.log(`Found ${documents ? documents.length : 0} documents in project`); // Add log

				if (documents && documents.length > 0) {
					projectDocuments = documents;
					currentDocumentIndex = projectDocuments.findIndex((doc) => doc.id.toString() === documentId);
					// Pre-populate the map for faster switching
					projectDocumentsMap.clear(); // Clear previous map entries
					projectDocuments.forEach((doc) => {
						projectDocumentsMap.set(doc.id, doc);
					});
					// Ensure project name is set if available
					if(documentData) documentData.project_name = projectInfo.project_name;
					console.log(`projectDocuments array updated. Current index: ${currentDocumentIndex}`); // Add log
				} else {
					// Handle case where project exists but has no documents (or only the current one)
					projectDocuments = documentData ? [documentData] : []; // Show at least current doc if available
					currentDocumentIndex = 0;
					projectDocumentsMap.clear();
					if(documentData) projectDocumentsMap.set(documentData.id, documentData);
					console.log('Project has no other documents, showing only current.'); // Add log
				}
			} else {
				// Document is not part of a project
				projectDocuments = documentData ? [documentData] : [];
				currentDocumentIndex = 0;
				projectDocumentsMap.clear();
				if(documentData) projectDocumentsMap.set(documentData.id, documentData);
				console.log('Document is not part of a project.'); // Add log
			}
			projectDocumentsLoaded = true;
			console.log('Finished loadProjectDocuments'); // Add log
		} catch (error) {
			console.error('Error loading project documents:', error);
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

	function exitInsertMode() {
		editorMode = 'NORMAL';
		showCommandError('NORMAL'); // Optional feedback
		clearNormalModeBuffer(); // Clear any pending sequence
	}

	// Function to handle command input
	function handleCommandInput() {}

	// Function to show command error for a few seconds
	function showCommandError(message: string) {
		commandError = message;
		commandErrorTimeout = setTimeout(() => {
			commandError = '';
		}, 3000);
	}

	type ToastData = {
		message: string;
		type: 'success' | 'error' | 'warning';
	};

	let toasts: ToastData[] = [];

	// --- Keybinding variables for Cheat Sheet --- 
	let boldKey = 'Ctrl+B';
	let italicKey = 'Ctrl+I';
	let underlineKey = 'Ctrl+U';
	let colorPickerKey = 'Ctrl+F';
	let insertModeKey = 'I';
	let moveLeftKey = 'H';
	let moveRightKey = 'L';
	let moveUpKey = 'K';
	let moveDownKey = 'J';
	let startOfLineKey = '0';
	let endOfLineKey = 'Shift+$';
	let endOfDocKey = 'Shift+G';
	let startOfDocKey = 'g';
	let toggleSheetKey = 'Ctrl+/';
	let nextMatchKey = 'N';
	let prevMatchKey = 'M';
	let deleteSelectedKey = 'X';
	let yankKey = 'Y';
	let deleteLineKey = 'D';
	let pasteKey = 'P';
	let switchDoc1Key = 'Ctrl+1';
	let switchDoc2Key = 'Ctrl+2';
	let switchDoc3Key = 'Ctrl+3';
	let switchDoc4Key = 'Ctrl+4';
	let switchDoc5Key = 'Ctrl+5';
	let switchDoc6Key = 'Ctrl+6';
	let switchDoc7Key = 'Ctrl+7';
	let switchDoc8Key = 'Ctrl+8';
	let switchDoc9Key = 'Ctrl+9';
	let toggleChatKey = 'Alt + C';

	function formatKey(input: KeyboardInput): string {
		let parts: string[] = [];
		if (!input) return ''; // Add check for undefined input

		if (input.ctrlDown) parts.push('Ctrl');
		if (input.altDown) parts.push('Alt');
		if (input.shiftDown) parts.push('Shift');
		
		// Use kd property instead of keyDown
		let key = input.kd; 
		if (!key) return parts.join('+'); // Return early if key is missing

		if (key.length === 1) key = key.toUpperCase(); 
		// Handle specific key names if needed
		parts.push(key); 
		return parts.join('+');
	}

	// Updated function to populate specific key variables
	function prepareCommandSheetData() {
		const bindings = keybindings.activeBindings;
		boldKey = bindings.bold ? formatKey(bindings.bold) : boldKey;
		italicKey = bindings.italic ? formatKey(bindings.italic) : italicKey;
		underlineKey = bindings.underline ? formatKey(bindings.underline) : underlineKey;
		colorPickerKey = bindings.openColorPicker ? formatKey(bindings.openColorPicker) : colorPickerKey;
		insertModeKey = bindings.enterInsertMode ? formatKey(bindings.enterInsertMode) : insertModeKey;
		moveLeftKey = bindings.moveLeft ? formatKey(bindings.moveLeft) : moveLeftKey;
		moveRightKey = bindings.moveRight ? formatKey(bindings.moveRight) : moveRightKey;
		moveUpKey = bindings.moveUp ? formatKey(bindings.moveUp) : moveUpKey;
		moveDownKey = bindings.moveDown ? formatKey(bindings.moveDown) : moveDownKey;
		startOfLineKey = bindings.moveToStartOfLine ? formatKey(bindings.moveToStartOfLine) : startOfLineKey;
		endOfLineKey = bindings.moveToEndOfLine ? formatKey(bindings.moveToEndOfLine) : endOfLineKey;
		endOfDocKey = bindings.moveToEndOfDocument ? formatKey(bindings.moveToEndOfDocument) : endOfDocKey;
		startOfDocKey = bindings.moveToStartOfDocument ? formatKey(bindings.moveToStartOfDocument) : startOfDocKey;
		toggleSheetKey = bindings.toggleCommandSheet ? formatKey(bindings.toggleCommandSheet) : toggleSheetKey;
		nextMatchKey = bindings.findNextMatch ? formatKey(bindings.findNextMatch) : nextMatchKey;
		prevMatchKey = bindings.findPreviousMatch ? formatKey(bindings.findPreviousMatch) : prevMatchKey;
		deleteSelectedKey = bindings.deleteSelectedText ? formatKey(bindings.deleteSelectedText) : deleteSelectedKey;
		yankKey = bindings.yankText ? formatKey(bindings.yankText) : yankKey;
		deleteLineKey = bindings.deleteLine ? formatKey(bindings.deleteLine) : deleteLineKey;
		pasteKey = bindings.pasteText ? formatKey(bindings.pasteText) : pasteKey;
		// Add document switch keys
		switchDoc1Key = bindings.switchToDocument1 ? formatKey(bindings.switchToDocument1) : switchDoc1Key;
		switchDoc2Key = bindings.switchToDocument2 ? formatKey(bindings.switchToDocument2) : switchDoc2Key;
		switchDoc3Key = bindings.switchToDocument3 ? formatKey(bindings.switchToDocument3) : switchDoc3Key;
		switchDoc4Key = bindings.switchToDocument4 ? formatKey(bindings.switchToDocument4) : switchDoc4Key;
		switchDoc5Key = bindings.switchToDocument5 ? formatKey(bindings.switchToDocument5) : switchDoc5Key;
		switchDoc6Key = bindings.switchToDocument6 ? formatKey(bindings.switchToDocument6) : switchDoc6Key;
		switchDoc7Key = bindings.switchToDocument7 ? formatKey(bindings.switchToDocument7) : switchDoc7Key;
		switchDoc8Key = bindings.switchToDocument8 ? formatKey(bindings.switchToDocument8) : switchDoc8Key;
		switchDoc9Key = bindings.switchToDocument9 ? formatKey(bindings.switchToDocument9) : switchDoc9Key;
		toggleChatKey = bindings.toggleChatAssistant ? formatKey(bindings.toggleChatAssistant) : toggleChatKey; // Update chat toggle key
	}

	// Function to show a toast notification
	function showToast(message: string, type: 'success' | 'error' | 'warning' = 'success') {
		toasts = [...toasts, { message, type }];
		// Remove the toast after 3 seconds
		setTimeout(() => {
			toasts = toasts.filter((t) => t.message !== message);
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
				update_document(documentData)
					.then(() => {
						showToast('Document saved successfully', 'success');
					})
					.catch((error) => {
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
				update_document(documentData)
					.then(() => {
						showToast('Document saved successfully', 'success');
					goto('/drive');
					})
					.catch((error) => {
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
			// Remove the '%s/' prefix and split the remaining command
			const commandText = cmd.substring(3); // Remove '%s/'

			// Find all forward slashes
			const slashPositions = [];
			let pos = -1;
			while ((pos = commandText.indexOf('/', pos + 1)) !== -1) {
				slashPositions.push(pos);
			}

			// We need at least 2 slashes for a valid command
			if (slashPositions.length >= 2) {
				// Extract search and replace terms
				const searchText = commandText.substring(0, slashPositions[0]);
				const replaceText = commandText.substring(slashPositions[0] + 1, slashPositions[1]);
				const flags = commandText.substring(slashPositions[1] + 1);

				console.log('Find and replace:', {
					search: searchText,
					replace: replaceText,
					flags
				});

				if (searchText && editorElement) {
					try {
						// Create flags for the regex - default to global if no flags specified
						const isGlobal = flags.includes('g') || flags === '';
						const isCaseInsensitive = flags.includes('i');
					const regexFlags = (isGlobal ? 'g' : '') + (isCaseInsensitive ? 'i' : '');

						// Escape special regex characters in search text if not using regex
						const escapedSearchText = flags.includes('r')
							? searchText
							: searchText.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

						console.log('Creating regex with:', {
							pattern: escapedSearchText,
							flags: regexFlags
						});

						// Create search regex
						const searchRegex = new RegExp(escapedSearchText, regexFlags);

						// Get the content as an array of lines
						const divs = Array.from(editorElement.querySelectorAll('div'));
						const originalLines = divs.map((div) => div.textContent || '');

						// Track replacements
						let totalReplacements = 0;
						const newLines = originalLines.map((line, index) => {
							// Count matches in this line before replacement
							const matches = line.match(searchRegex);
							if (matches) {
								totalReplacements += matches.length;
							}

							// Reset lastIndex for global regex between lines
							searchRegex.lastIndex = 0;
					
					// Perform the replacement
							return line.replace(searchRegex, replaceText);
						});

						if (totalReplacements > 0) {
							// Update the content of each div
							divs.forEach((div, index) => {
								if (index < newLines.length) {
									// Create a text node with the new content
									const textNode = document.createTextNode(newLines[index]);
									// Clear the div and add the new text node
									div.textContent = '';
									div.appendChild(textNode);
								}
							});

							// Update editor content
							editorContent = newLines.join('\n');

							// Update document data
					if (documentData) {
								documentData.content = editorElement.innerHTML;
							}

							// Show success message with the actual search text
							showCommandError(
								`Replaced ${totalReplacements} occurrence${totalReplacements !== 1 ? 's' : ''} of "${searchText}"`
							);

							// Update UI
							updateLineNumbers();
							updateCursorPosition();
							adjustEditorHeight();

							return true;
				} else {
							showCommandError(`No matches found for "${searchText}"`);
							return false;
						}
					} catch (e) {
						console.error('Error in find and replace:', e);
						showCommandError('Invalid search pattern');
						return false;
				}
			} else {
					showCommandError('Invalid find and replace syntax. Use :%s/search/replace/[flags]');
					return false;
				}
			} else {
				showCommandError('Invalid find and replace syntax. Use :%s/search/replace/[flags]');
				return false;
			}
		} else {
			// Show error for unrecognized command
			showCommandError(`Unknown command: "${command}"`);
			return false;
		}

		return true;
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
			let emptyDivs = currentDivs.filter((div) => (div.textContent || '').trim() === '');
			
			// Only remove empty divs if they're not the only div
			if (emptyDivs.length > 0 && emptyDivs.length < currentDivs.length) {
				emptyDivs.forEach((div) => {
					div.remove();
				});
			}
			
			// Get updated content after removal
			editorContent = getEditorContent();
			
			// Try to restore a reasonable cursor position
			const newContentLength = editorContent.length;
			const safePosition = Math.min(start, newContentLength);
			setRange(editorElement, safePosition, safePosition);
		} else {
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
				const emptyDivs = allDivs.filter((div) => (div.textContent || '').trim() === '');
				
				// Remove empty divs if there are other non-empty divs
				if (emptyDivs.length > 0 && emptyDivs.length < allDivs.length) {
					emptyDivs.forEach((div) => {
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
		console.log(`KeyDown event: {key: '${event.key}', ctrlKey: ${event.ctrlKey}, metaKey: ${event.metaKey}, shiftKey: ${event.shiftKey}, showColorPicker: ${showColorPicker}}`);

		// Log the currently active binding for underline for debugging
		console.log('Active underline binding:', keybindings.activeBindings.underline);

		if (showColorPicker) {
			handleColorPickerKeyDown(event);
			return;
		}

		preventBrowserDefaults(event);

		// If color picker is open, handle its navigation
		if (showColorPicker) {
			console.log('Color picker is open, handling navigation');
			if (event.key === 'Escape') {
				event.preventDefault();
				console.log('Closing color picker');
				showColorPicker = false;
				return;
			}
			if (event.key === 'Enter') {
				event.preventDefault();
				console.log('Applying color');
				applyTextColor(selectedColor);
				showColorPicker = false;
				return;
			}
			if (event.key === 'ArrowLeft' || event.key === 'h') {
				event.preventDefault();
				hue = (hue - 5 + 365) % 365; // Wrap around when going below 0
				updateColorFromHueOnly();
				return;
			}
			if (event.key === 'ArrowRight' || event.key === 'l') {
				event.preventDefault();
				hue = (hue + 5) % 365; // Wrap around when exceeding 365
				updateColorFromHueOnly();
				return;
			}
			if (event.key === 'ArrowUp' || event.key === 'k') {
				event.preventDefault();
				hue = (hue + 15) % 365; // Larger jump up with wrap around
				updateColorFromHueOnly();
				return;
			}
			if (event.key === 'ArrowDown' || event.key === 'j') {
				event.preventDefault();
				hue = (hue - 15 + 365) % 365; // Larger jump down with wrap around
				updateColorFromHueOnly();
				return;
			}
			// Block all other keys while color picker is open
			event.preventDefault();
			return;
		}

		// Handle Escape key to exit any mode and return to NORMAL mode
		if (event.key === 'Escape') {
			editorMode = 'NORMAL';
			clearNormalModeBuffer();
			event.preventDefault();
			return;
		}

		// COMMAND MODE: Handle only Escape (already done) and Enter
		if (editorMode === 'COMMAND') {
			if (event.key === 'Enter') {
				// Command execution is handled by the executeCommand function
				return;
			}
			clearNormalModeBuffer();
			return;
		}

		// INSERT MODE: Delegate to the specific handler
		if (editorMode === 'INSERT') {
			handleInsertModeKeyDown(event);
			// After handling specific keys (like Escape or formatting), 
			// update UI for regular typing
			setTimeout(() => {
				updateCursorPosition();
				updateLineNumbers();
				ensureCursorVisible();
			}, 0);
			return; // Return after handling INSERT mode
		}

		// NORMAL MODE: Handle editor commands
		if (editorMode === 'NORMAL') {
			// Clear any existing buffer timeout
			if (normalModeBufferTimeout) {
					clearTimeout(normalModeBufferTimeout);
					normalModeBufferTimeout = null;
			}

			// Allow text selection with Shift + arrow keys in normal mode
			if (
				event.shiftKey &&
				(event.key === 'ArrowLeft' ||
					event.key === 'ArrowRight' ||
					event.key === 'ArrowUp' ||
					event.key === 'ArrowDown' ||
					event.key === 'h' ||
					event.key === 'j' ||
					event.key === 'k' ||
					event.key === 'l')
			) {
				return; // Let browser handle selection
			}

			if (event.key === 'ArrowLeft') {
				event.preventDefault();
				moveLeft();
				return;
			}
			if (event.key === 'ArrowRight') {
				event.preventDefault();
				moveRight();
				return;
			}
			if (event.key === 'ArrowUp') {
				event.preventDefault();
				moveUp();
				return;
			}
			if (event.key === 'ArrowDown') {
				event.preventDefault();
				moveDown();
				return;
			}

			if (event.key === '/' && !event.ctrlKey && !event.shiftKey) {
				event.preventDefault();
				enterCommandMode('/'); // Enter command mode with forward search prefix
				return;
			}
			if (event.key === '?' && !event.ctrlKey) {
				event.preventDefault();
				enterCommandMode('?'); // Enter command mode with backward search prefix
				return;
			}

			if (event.key === ':') {
				event.preventDefault();
				enterCommandMode(':'); // Enter command mode with colon prefix
				return;
			}

			// For any other key in NORMAL mode that wasn't handled above 
			// or by the keybinding system (which runs separately via window listener),
			// prevent the default browser action (e.g., inserting characters).
			// Exception: Allow Ctrl+A (Select All) default behavior.
			if (!( (event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'a') ) {
				event.preventDefault(); 
			}

		}
	}

	// Add deleteHighlightedText function
	function deleteHighlightedText() {
		if (!editorElement) return;

		const selection = window.getSelection();
		if (!selection || !selection.rangeCount) return;

		const range = selection.getRangeAt(0);
		const currentDiv = range.startContainer.parentElement;

		// If no text is selected, delete the character at cursor
		if (range.collapsed) {
			const textNode = range.startContainer;
			if (textNode.nodeType === Node.TEXT_NODE) {
				const offset = range.startOffset;
				const text = textNode.textContent || '';

				// Only proceed if there's a character to delete
				if (offset < text.length) {
					const newText = text.slice(0, offset) + text.slice(offset + 1);
					textNode.textContent = newText;

					// Maintain cursor position
					range.setStart(textNode, offset);
					range.setEnd(textNode, offset);
					selection.removeAllRanges();
					selection.addRange(range);
				}
			}
		} else {
			// Delete selected text
			range.deleteContents();
		}

		// Update editor content
		editorContent = getEditorContent();

		// Update UI
				updateCursorPosition();
				updateLineNumbers();
		adjustEditorHeight();

		// Show feedback
		showCommandError('Text deleted');
	}

	// Update the updateCursorPosition function to ensure all lines (including empty ones) are properly highlighted
	function updateCursorPosition() {
		if (!editorElement || !browser) return;

		console.log('Updating cursor position');
		
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
				if (div.contains(textNode) || (textNode === editorElement && offset >= i && offset <= i + 1)) {
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

					console.log('Cursor position updated:', {
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

			console.log('Cursor position updated:', {
				activeLineIndex,
				cursorLine,
				cursorColumn,
				totalLines: allDivs.length
			});
		} catch (error) {
			console.error('Error updating cursor position:', error);
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
			if (range.startContainer === editorElement && range.startOffset === editorElement.childNodes.length) {
					// Cursor is after the last div, add an extra line
					numberOfLines++;
				}
		}
		
		// Calculate the height based on number of lines
		const contentHeight = numberOfLines * LINE_HEIGHT;
		
		// Calculate the minimum height based on MIN_LINES
		const minHeight = LINE_HEIGHT * MIN_LINES;

		// Set height to the larger of content height or minimum height, plus extra padding
		const newHeight = Math.max(contentHeight, minHeight) + 48; // Add extra padding

		// Apply the new height to editor without overflow
		editorElement.style.height = `${newHeight}px`;
		
		// Remove overflow scrolling from editor element - we want page scrolling, not editor scrolling
		editorElement.style.overflowY = 'visible';
		editorElement.style.maxHeight = 'none'; // Remove max height constraint

		// Update the line numbers container height to match
		const lineNumbersContainer = document.querySelector('.line-numbers') as HTMLElement;
		if (lineNumbersContainer) {
			lineNumbersContainer.style.height = `${newHeight}px`;
		}

		// Force a reflow to ensure the height is applied
		editorElement.offsetHeight;

		// Ensure cursor is visible after height adjustment
		ensureCursorVisible();
	}

	// Add this function to ensure cursor is visible
	function ensureCursorVisible() {
		if (!editorElement) return;
		
		const selection = window.getSelection();
		if (!selection || !selection.rangeCount) return;
		
		const range = selection.getRangeAt(0);
		const rect = range.getBoundingClientRect();
		
		// Check if cursor is visible in viewport
		const isVisible =
			rect.top >= 0 &&
			rect.left >= 0 &&
			rect.bottom <= (window.innerHeight || document.documentElement.clientHeight) &&
			rect.right <= (window.innerWidth || document.documentElement.clientWidth);
		
		if (!isVisible) {
			// Scroll to make cursor visible
			window.scrollTo({
				top: window.scrollY + rect.top - window.innerHeight / 2,
				behavior: 'smooth'
			});
		}
	}

	// Update the onMount function for proper initialization
	onMount(async () => {
		// Check if we're in a browser environment first
		if (!browser) return;

		console.log('onMount: Initializing editor');
		
		// Load document data and profile image in parallel
		try {
			const [docResult, userResult] = await Promise.all([loadDocumentData(), loadUserProfile()]);

			console.log('onMount: Document loaded, content length:', editorContent?.length || 0);
			
			// Set navbar ready first
			navbarReady = true;
			
			// Then set document ready with a shorter delay
			setTimeout(() => {
				documentReady = true;
				
				// Force refresh line numbers and layout after everything is loaded
				setTimeout(() => {
					// Set initial cursor and focus the editor
					if (editorElement) {
						console.log('Editor element found in onMount:', editorElement);

						// Ensure content is set
						if (editorContent && (!editorElement.innerHTML || editorElement.innerHTML.trim() === '')) {
							console.log('Editor was empty, setting content from editorContent');
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

		await keybindings.fetchAndUpdateBindings();
		// Update the command sheet data after bindings are loaded
		prepareCommandSheetData();
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
			console.log('Document data loaded:', documentData);

			if (documentData) {
				// Get the HTML content from the document
				let htmlContent = documentData.content || '';
				console.log('Raw HTML content:', htmlContent);

				// If the content is empty, create a default empty div
				if (htmlContent.trim() === '') {
					htmlContent = '<div><br></div>';
				}

				// If content doesn't have div tags, convert it to HTML format
				if (!htmlContent.includes('<div')) {
					// Convert plain text lines to HTML with divs
					const lines = htmlContent.split('\n');
					htmlContent = lines
						.map((line: string) => {
							// If line is empty, add a <br> tag to preserve it
							return line.trim() === '' ? '<div><br></div>' : `<div>${line}</div>`;
						})
						.join('');
					console.log('Converted to HTML:', htmlContent);
				}

				// If content doesn't have div tags, convert it to HTML format
				if (!htmlContent.includes('<div')) {
					// Convert plain text lines to HTML with divs
					const lines = htmlContent.split('\n');
					htmlContent = lines
						.map((line: string) => {
							// If line is empty, add a <br> tag to preserve it
							return line.trim() === '' ? '<div><br></div>' : `<div>${line}</div>`;
						})
						.join('');
				}

				// Store the HTML content
				editorContent = htmlContent;
				console.log('Editor content set to:', editorContent);

				// Set the content in the editor
				if (editorElement) {
					// Directly set innerHTML to render the HTML structure
					console.log('Setting innerHTML on editorElement:', htmlContent);
					editorElement.innerHTML = htmlContent;
				} else {
					console.warn('Editor element not available yet for setting content');
					// Set up a retry mechanism for when the editor element becomes available
					const maxRetries = 5;
					let retryCount = 0;

					const retryInterval = setInterval(() => {
						if (editorElement) {
							console.log('Editor element now available, setting content');
							editorElement.innerHTML = htmlContent;
							clearInterval(retryInterval);

							// Update UI after setting content
							updateLineNumbers();
							updateCursorPosition();
							adjustEditorHeight();
						} else if (retryCount >= maxRetries) {
							console.error('Failed to set editor content after max retries');
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

				// Re-add Set up auto-save
				autoSaveCleanup = setup_auto_save(documentData, () => {
					if (documentData && editorElement) {
						// Save the inner HTML of the editor element
						documentData.content = editorElement.innerHTML;
						console.log('Auto-saving content:', editorElement.innerHTML);
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
				continue;
			}

				// Line is too long and needs wrapping
				let remainingText = line;
				
			while (remainingText.length > 0) {
				// If remaining text fits in one line
				if (remainingText.length <= MAX_COLUMN_WIDTH) {
					if (remainingText.trim()) {
						wrappedLines.push(remainingText);
					}
					break;
				}

				// Always try to break at a space within the limit first
				let breakIndex = -1;
				for (let i = MAX_COLUMN_WIDTH; i >= MAX_COLUMN_WIDTH - 20; i--) {
					if (remainingText[i] === ' ') {
						breakIndex = i;
						break;
					}
				}

				// If no suitable space found, force break at MAX_COLUMN_WIDTH
				if (breakIndex === -1) {
						breakIndex = MAX_COLUMN_WIDTH;
					}
					
				// Get the segment up to the break point, ensuring it's not longer than MAX_COLUMN_WIDTH
				const segment = remainingText.substring(0, Math.min(breakIndex, MAX_COLUMN_WIDTH)).trimEnd();
				if (segment) {
					wrappedLines.push(segment);
				}

				// Move to next segment, trimming any leading spaces
					remainingText = remainingText.substring(breakIndex).trimStart();
			}
		}

		// Ensure we always have at least one line
		if (wrappedLines.length === 0) {
			wrappedLines.push('');
		}

		// Double-check that no line exceeds MAX_COLUMN_WIDTH
		return wrappedLines
			.map((line) => {
				if (line.length > MAX_COLUMN_WIDTH) {
					return line.substring(0, MAX_COLUMN_WIDTH);
				}
				return line;
			})
			.join('\n');
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
			const lines = Array.from(divElements).map((div) => {
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
				if (range.startContainer === editorElement && range.startOffset === editorElement.childNodes.length) {
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
		console.log(
			`getEditorContent: ${content.split('\n').length} lines (${content.split('\n').filter((l) => l === '').length} empty)`
		);
		
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
		
		let totalOffset = 0;
		const divs = Array.from(editorElement.querySelectorAll('div'));

		for (let i = 0; i < divs.length; i++) {
			const div = divs[i];
			if (div.contains(node)) {
				// Found the div containing our node
				let currentNode = div.firstChild;
				while (currentNode && currentNode !== node) {
					if (currentNode.nodeType === Node.TEXT_NODE) {
						totalOffset += currentNode.textContent?.length || 0;
					}
					currentNode = currentNode.nextSibling;
				}
				return totalOffset + offset;
			} else {
				// Add length of previous divs
				totalOffset += div.textContent?.length || 0;
				// Only add newline if not the last div
				if (i < divs.length - 1) {
					totalOffset += 1;
				}
			}
		}

		return totalOffset;
	}

	// Helper function to set cursor position by character offset
	function setCursorPositionByOffset(offset: number) {
		if (!editorElement) return;
		
		let currentPos = 0;
		const divs = Array.from(editorElement.querySelectorAll('div'));

		for (let i = 0; i < divs.length; i++) {
			const div = divs[i];
			const divLength = (div.textContent || '').length + 1; // +1 for newline

			if (currentPos + divLength > offset) {
				// Found the div containing our target position
				const offsetInDiv = offset - currentPos;
				let currentNode = div.firstChild;
		let currentOffset = 0;
		
		while (currentNode) {
					if (currentNode.nodeType === Node.TEXT_NODE) {
						const nodeLength = currentNode.textContent?.length || 0;
						if (currentOffset + nodeLength >= offsetInDiv) {
				const range = document.createRange();
							range.setStart(currentNode, offsetInDiv - currentOffset);
				range.collapse(true);
				
							const selection = window.getSelection();
							if (selection) {
								selection.removeAllRanges();
								selection.addRange(range);
							}
				return;
			}
			currentOffset += nodeLength;
					}
					currentNode = currentNode.nextSibling;
				}
				break;
			}
			currentPos += divLength;
		}
	}


	function applyTextColor(color: string) {
		if (document.queryCommandSupported('foreColor')) {
			// Save the current selection
			const selection = window.getSelection();
			if (!selection || !selection.rangeCount) return;
			
			const range = selection.getRangeAt(0);
			
			// Check if the selection is within an underlined element
			let isWithinUnderline = false;
			let parentNode: Node | null = range.commonAncestorContainer;
			
			// Walk up the DOM tree to check for u tag
			while (parentNode && parentNode !== editorElement) {
				if (parentNode.nodeName === 'U') {
					isWithinUnderline = true;
					break;
				}
				if (parentNode.parentNode) {
					parentNode = parentNode.parentNode;
				} else {
					break;
				}
			}
			
			// Apply the color formatting
			document.execCommand('foreColor', false, color);
			
			// If the selection was within an underline element, fix the structure
			if (isWithinUnderline && editorElement) {
				// Get the updated selection after color change
				const updatedSelection = window.getSelection();
				if (!updatedSelection || !updatedSelection.rangeCount) return;
				
				const updatedRange = updatedSelection.getRangeAt(0);
				const fragment = updatedRange.cloneContents();
				
				// Find any font elements that are direct children of u elements
				const fontElements = editorElement.querySelectorAll('u > font');
				for (const fontElement of fontElements) {
					const fontColor = fontElement.getAttribute('color');
					const fontContent = fontElement.innerHTML;
					const parentNode = fontElement.parentNode;
					
					// Replace with the correct structure: font > u
					if (fontColor && parentNode && parentNode.nodeName === 'U') {
						const newElement = document.createElement('font');
						newElement.setAttribute('color', fontColor);
						
						const newUnderline = document.createElement('u');
						newUnderline.innerHTML = fontContent;
						
						newElement.appendChild(newUnderline);
						parentNode.replaceChild(newElement, fontElement);
					}
				}
			}
			
			showCommandError(`Text color changed to ${color}`);
		} else {
			showCommandError('Text color formatting not supported');
		}
		
		// Ensure editor remains focused after applying color
		if(editorElement) {
			editorElement.focus();
			// Update internal content state AFTER execCommand modifies the DOM
			editorContent = getEditorContent(); 
			// Explicitly update UI elements that might rely on content length/lines
			updateLineNumbers();
			adjustEditorHeight();
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
		if (!editorElement || !event.clipboardData) return;
		
		console.log('Paste event triggered');
		event.preventDefault(); // Prevent default paste

		// Get plain text from clipboard
		let clipboardText = event.clipboardData.getData('text/plain');
		console.log('Raw clipboard text:', clipboardText);

		// Normalize line endings
		clipboardText = clipboardText.replace(/\r\n/g, '\n');

		// Get current selection and offset
		const selection = window.getSelection();
		if (!selection || !selection.rangeCount) return;
		const range = selection.getRangeAt(0);
		const startOffsetInDocument = getTextOffset(range.startContainer, range.startOffset);
		
		console.log('Paste position:', {
			container: range.startContainer,
			offset: range.startOffset,
			documentOffset: startOffsetInDocument
		});
		
		// If text is selected, delete it first
		if (!range.collapsed) {
			range.deleteContents();
		}
		
		// Get current editor content
		let currentContent = getEditorContent();
		
		// Insert clipboard text into content string at the correct offset
		const contentBefore = currentContent.substring(0, startOffsetInDocument);
		const contentAfter = currentContent.substring(startOffsetInDocument);
		const newContent = contentBefore + clipboardText + contentAfter;
		const insertedLength = clipboardText.length;
		const targetOffset = startOffsetInDocument + insertedLength;

		// Use safelySetEditorContent to handle wrapping and update UI
		safelySetEditorContent(newContent);
		
		setTimeout(() => {
			console.log(`Restoring cursor position after paste to offset: ${targetOffset}`);
			setCursorPositionByOffset(targetOffset);
			// Also ensure updates run again after setting cursor
			updateLineNumbers();
			ensureCursorVisible();
			adjustEditorHeight();
		}, 0); // Use setTimeout to ensure DOM is updated by safelySetEditorContent
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
		
		console.log('Moving to end of line');

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
			console.log('Found current div:', currentDiv);

			// Create a new range at the end of the div
			const newRange = document.createRange();

			// Get all text nodes in the div
			const textNodes: Node[] = [];
			const walker = document.createTreeWalker(currentDiv, NodeFilter.SHOW_TEXT, null);

			let node;
			while ((node = walker.nextNode())) {
				textNodes.push(node);
			}

			// If we have text nodes, move to the end of the last one
			if (textNodes.length > 0) {
				const lastTextNode = textNodes[textNodes.length - 1];
				const length = lastTextNode.textContent?.length || 0;
				newRange.setStart(lastTextNode, length);

				// Calculate total length for cursor column
				let totalLength = 0;
				textNodes.forEach((node) => {
					totalLength += node.textContent?.length || 0;
				});
				cursorColumn = totalLength + 1;

				console.log('Moving to end of text:', {
					totalLength,
					cursorColumn,
					textContent: currentDiv.textContent
				});
			} else {
				// If no text nodes, set to end of div
				newRange.setStart(currentDiv, currentDiv.childNodes.length);
				cursorColumn = 1;
			}

			newRange.collapse(true);

			// Apply the range
			selection.removeAllRanges();
			selection.addRange(newRange);

			// Update UI
			updateCursorPosition();
			updateLineNumbers();
			ensureCursorVisible();
		}
	}
	
	// Function to move to the end of the document (G command)
	function moveToEndOfDocument() {
		if (!editorElement) return;
		
		console.log('Moving to end of document');
		
			// For contenteditable divs, use DOM approach
			const allDivs = Array.from(editorElement.querySelectorAll('div'));
			
			if (allDivs.length > 0) {
				// Get the last div
				const lastDiv = allDivs[allDivs.length - 1];

			// Get all text nodes in the last div
			const textNodes: Node[] = [];
			const walker = document.createTreeWalker(lastDiv, NodeFilter.SHOW_TEXT, null);

			let node;
			while ((node = walker.nextNode())) {
				textNodes.push(node);
			}
				
				// Create a range at the end of the last div
				const range = document.createRange();
				
			// If we have text nodes, move to the end of the last one
			if (textNodes.length > 0) {
				const lastTextNode = textNodes[textNodes.length - 1];
				const length = lastTextNode.textContent?.length || 0;
				range.setStart(lastTextNode, length);

				// Calculate total length for cursor column
				let totalLength = 0;
				textNodes.forEach((node) => {
					totalLength += node.textContent?.length || 0;
				});
				cursorColumn = totalLength + 1;
				} else {
				// If no text nodes, set to end of div
				range.setStart(lastDiv, lastDiv.childNodes.length);
				cursorColumn = 1;
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
				
			console.log('Moved to end of document:', {
				activeLineIndex,
				cursorLine,
				cursorColumn,
				totalLines: allDivs.length,
				lastLineContent: lastDiv.textContent || ''
			});
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
			} else if (commandPrefix === '/' || commandPrefix === '?') {
				// Set the search direction based on the command
				const direction = commandPrefix === '/' ? 'forward' : 'backward';
				performSearch(commandInput, direction);
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
		
		console.log('Setting content safely:', content);

		// REMOVED block that stripped HTML tags - Assume input 'content' 
		// already represents lines separated by '\n'.
		// let plainContent = content;
		// if (content.includes('<') && content.includes('>')) { ... }

		// Split into lines directly from input content and wrap if needed
		let lines = content.split('\n');
		let wrappedLines: string[] = [];
		let originalLineToWrappedMap = new Map<number, number>(); // Maps original line index to first wrapped line index
		let wrappedToOriginalMap = new Map<number, number>(); // Maps wrapped line index back to original line
		let currentWrappedIndex = 0;

		for (let i = 0; i < lines.length; i++) {
			let line = lines[i];
			// Store mapping from original line to its first wrapped line
			originalLineToWrappedMap.set(i, currentWrappedIndex);

			if (line.length <= MAX_COLUMN_WIDTH) {
				wrappedLines.push(line);
				wrappedToOriginalMap.set(currentWrappedIndex, i);
				currentWrappedIndex++;
			} else {
				console.log('Line exceeds MAX_COLUMN_WIDTH, wrapping:', line.length);
				// Process the line in chunks
				while (line.length > 0) {
					let breakPoint = -1;

					if (line.length > MAX_COLUMN_WIDTH) {
						// Try to find a space to break at
						for (let j = MAX_COLUMN_WIDTH; j >= MAX_COLUMN_WIDTH - 20; j--) {
							if (line[j] === ' ') {
								breakPoint = j;
								break;
							}
						}

						// If no space found, force break at MAX_COLUMN_WIDTH
						if (breakPoint === -1) {
							breakPoint = MAX_COLUMN_WIDTH;
						}

						// Add the segment and continue with remaining text
						const segment = line.substring(0, breakPoint).trimEnd();
						wrappedLines.push(segment);
						wrappedToOriginalMap.set(currentWrappedIndex, i);
						currentWrappedIndex++;
						console.log('Created wrapped line:', segment.length);

						// Continue with remaining text
						line = line.substring(breakPoint).trimStart();
					} else {
						// Add remaining text if any
						if (line.length > 0) {
							wrappedLines.push(line);
							wrappedToOriginalMap.set(currentWrappedIndex, i);
							currentWrappedIndex++;
							console.log('Added final segment:', line.length);
						}
						break;
					}
				}
			}
		}

		console.log('Total wrapped lines:', wrappedLines.length);

		// Store reference to editor element to ensure it's not null during operations
		const editor = editorElement;

		// Clear the editor
		editor.innerHTML = '';

		// Create a div for each line
		wrappedLines.forEach((line, index) => {
			const div = document.createElement('div');

			// Handle empty lines
			if (line.trim() === '') {
				div.innerHTML = '<br>';
			} else {
				div.textContent = line; // textContent automatically escapes HTML
			}

			// Add data attribute to track original line number
			div.dataset.originalLine = wrappedToOriginalMap.get(index)?.toString() || '0';

			editor.appendChild(div);
		});

		// Ensure at least one div exists
		if (wrappedLines.length === 0) {
			const div = document.createElement('div');
			div.innerHTML = '<br>';
			div.dataset.originalLine = '0';
			editor.appendChild(div);
		}

		// Update our content tracking
		editorContent = wrappedLines.join('\n');
		lines = wrappedLines;

		// Update active line index based on the mapping
		if (originalLineToWrappedMap.has(activeLineIndex)) {
			const newActiveLineIndex = originalLineToWrappedMap.get(activeLineIndex) || 0;
			activeLineIndex = newActiveLineIndex;
			cursorLine = newActiveLineIndex + 1;
			console.log('Updated active line index after wrapping:', {
				oldIndex: activeLineIndex,
				newIndex: newActiveLineIndex,
				cursorLine
			});
		}
		
		// Force browser to recognize empty divs 
		const emptyDivs = Array.from(editor.querySelectorAll('div')).filter((div) => !div.textContent);
		emptyDivs.forEach((div) => {
			if (!div.firstChild) {
				div.appendChild(document.createTextNode('\u200B')); // Zero-width space
			}
		});

		console.log('Final editor structure:', editor.innerHTML);

		// Force update of line numbers and cursor position
		updateLineNumbers();
		updateCursorPosition();
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
	};

	const toastError = {
		theme: {
			'--toastBackground': '#F56565',
			'--toastBarBackground': '#C53030'
		}
	};

	// Add search functions
	function performSearch(query: string, direction: 'forward' | 'backward' = 'forward') {
		if (!editorElement || !query) {
			console.log('Search aborted:', !editorElement ? 'No editor element' : 'Empty query');
			return;
		}

		console.log('Starting search:', {
			query,
			direction,
			contentLength: editorContent.length
		});

		// Save last search for 'n' and 'N' commands
		lastSearchQuery = query;
		lastSearchDirection = direction;
		searchDirection = direction;

		// Get the current cursor position
		const selection = window.getSelection();
		if (!selection || !selection.rangeCount) {
			console.log('Search aborted: No valid selection');
			return;
		}

		const range = selection.getRangeAt(0);
		const currentOffset = getTextOffset(range.startContainer, range.startOffset);

		// Clear previous results
		searchResults = [];
		currentSearchIndex = -1;

		// Create a regex from the query - case sensitive by default
		try {
			console.log('Creating search regex:', query);
			const regex = new RegExp(query, 'g'); // Remove 'i' flag to make it case sensitive
			let match;

			// Get the raw text content without extra newlines
			const searchContent = Array.from(editorElement.querySelectorAll('div'))
				.map((div, i, arr) => div.textContent + (i < arr.length - 1 ? '\n' : ''))
				.join('');

			// Find all matches in the content
			while ((match = regex.exec(searchContent)) !== null) {
				searchResults.push(match.index);
				console.log('Found match:', {
					index: match.index,
					matchedText: match[0],
					surroundingContext: searchContent.substring(
						Math.max(0, match.index - 10),
						Math.min(searchContent.length, match.index + match[0].length + 10)
					)
				});
			}

			if (searchResults.length > 0) {
				// Find the appropriate result based on search direction
				if (direction === 'forward') {
					currentSearchIndex = searchResults.findIndex((pos) => pos > currentOffset);
					if (currentSearchIndex === -1) {
						currentSearchIndex = 0; // Wrap to start
			}
		} else {
					currentSearchIndex = searchResults.findIndex((pos) => pos >= currentOffset) - 1;
					if (currentSearchIndex === -2) {
						currentSearchIndex = searchResults.length - 1; // Wrap to end
					}
				}

				// Navigate to the result
				navigateToSearchResult();
			} else {
				showCommandError(`No matches found for: ${query}`);
			}
		} catch (e) {
			console.error('Search regex error:', e);
			showCommandError('Invalid search pattern');
		}
	}

	function navigateToSearchResult() {
		if (!editorElement || searchResults.length === 0 || currentSearchIndex < 0) {
			console.log('Navigate aborted:', {
				hasEditor: !!editorElement,
				resultsCount: searchResults.length,
				currentIndex: currentSearchIndex
			});
			return;
		}

		const position = searchResults[currentSearchIndex];
		const query = lastSearchQuery;

		console.log('Navigating to result:', {
			position,
			queryLength: query.length,
			resultNumber: currentSearchIndex + 1,
			totalResults: searchResults.length,
			matchText: editorContent.substring(position, position + query.length)
		});

		// Find the div and position within the div for the match
		let currentPos = 0;
		let targetDiv: HTMLDivElement | null = null;
		let offsetInDiv = 0;

		const divs = Array.from(editorElement.querySelectorAll('div'));

		// Calculate cumulative lengths and find target div
		const divLengths: number[] = [];
		for (const div of divs) {
			divLengths.push(currentPos);
			currentPos += (div.textContent || '').length + 1; // +1 for newline
		}

		// Find the div containing our position
		for (let i = 0; i < divs.length; i++) {
			const nextPos = i < divs.length - 1 ? divLengths[i + 1] : currentPos;
			if (position >= divLengths[i] && position < nextPos) {
				targetDiv = divs[i];
				offsetInDiv = position - divLengths[i];
				activeLineIndex = i;
				console.log('Found target div:', {
					divIndex: i,
					divContent: divs[i].textContent,
					offsetInDiv,
					totalOffset: position,
					divStart: divLengths[i],
					divEnd: nextPos
				});
				break;
			}
		}

		// Special handling for last line
		if (!targetDiv && position >= divLengths[divLengths.length - 1]) {
			const lastIndex = divs.length - 1;
			targetDiv = divs[lastIndex];
			offsetInDiv = position - divLengths[lastIndex];
			activeLineIndex = lastIndex;
			console.log('Found match in last div:', {
				divIndex: lastIndex,
				divContent: targetDiv.textContent,
				offsetInDiv,
				totalOffset: position
			});
		}

		if (targetDiv) {
			// Create a range for the match
			const range = document.createRange();
			let currentNode = targetDiv.firstChild;
			let currentOffset = 0;

			// If no text nodes exist, create one
			if (!currentNode) {
				const textNode = document.createTextNode(targetDiv.textContent || '');
				targetDiv.appendChild(textNode);
				currentNode = textNode;
			}

			// Find or create the appropriate text node
			while (currentNode) {
				if (currentNode.nodeType === Node.TEXT_NODE) {
					const nodeLength = currentNode.textContent?.length || 0;
					if (currentOffset + nodeLength >= offsetInDiv) {
						// Found the node containing our position
						const startOffset = offsetInDiv - currentOffset;

						// Ensure we don't exceed the node's length
						const endOffset = Math.min(startOffset + query.length, nodeLength);

						console.log('Setting range:', {
							node: currentNode.textContent,
							startOffset,
							endOffset,
							nodeLength
						});

						try {
							range.setStart(currentNode, startOffset);
							range.setEnd(currentNode, endOffset);

							const selection = window.getSelection();
							if (selection) {
								selection.removeAllRanges();
								selection.addRange(range);

								// Force scroll into view if needed
								const rect = range.getBoundingClientRect();
								if (rect.top < 0 || rect.bottom > window.innerHeight) {
									targetDiv.scrollIntoView({ behavior: 'smooth', block: 'center' });
								}
							}
						} catch (e) {
							console.error('Error setting range:', e, {
								startOffset,
								endOffset,
								nodeLength,
								text: currentNode.textContent
							});
						}
						break;
					}
					currentOffset += nodeLength;
				}
				currentNode = currentNode.nextSibling;
			}
		}

		// Update cursor position and line numbers
		updateCursorPosition();
		updateLineNumbers();

		// Show feedback
		showCommandError(`Match ${currentSearchIndex + 1} of ${searchResults.length}`);
	}

	function findNextMatch(reverse = false) {
		if (!lastSearchQuery || searchResults.length === 0) {
			console.log('Next match aborted:', {
				hasLastQuery: !!lastSearchQuery,
				resultsCount: searchResults.length
			});
			showCommandError('No previous search');
			return;
		}

		console.log('Finding next match:', {
			reverse,
			currentIndex: currentSearchIndex,
			totalResults: searchResults.length,
			lastQuery: lastSearchQuery
		});

		if (reverse) {
			// Move backward through results
			currentSearchIndex--;
			if (currentSearchIndex < 0) {
				currentSearchIndex = searchResults.length - 1; // Wrap to end
				showCommandError(`Wrapped to bottom, match ${currentSearchIndex + 1} of ${searchResults.length}`);
			} else {
				showCommandError(`Match ${currentSearchIndex + 1} of ${searchResults.length}`);
			}
		} else {
			// Move forward through results
			currentSearchIndex++;
			if (currentSearchIndex >= searchResults.length) {
				currentSearchIndex = 0; // Wrap to beginning
				showCommandError(`Wrapped to top, match ${currentSearchIndex + 1} of ${searchResults.length}`);
			} else {
				showCommandError(`Match ${currentSearchIndex + 1} of ${searchResults.length}`);
			}
		}

		console.log('Selected next match:', {
			newIndex: currentSearchIndex,
			position: searchResults[currentSearchIndex],
			matchText: editorContent.substring(
				searchResults[currentSearchIndex],
				searchResults[currentSearchIndex] + lastSearchQuery.length
			)
		});

		navigateToSearchResult();
	}

	// Redesigned updateLineNumbers function for better empty line handling
	function updateLineNumbers() {
		if (!editorElement) return;

		const lineNumbersContainer = document.querySelector('.editor-content .line-numbers');
		if (!lineNumbersContainer) return;

		// Determine line count based on editor content
		let lineCount = 0;
		const divElements = Array.from(editorElement.querySelectorAll('div'));
		const divCount = divElements.length;

		if (divCount === 1) {
			// Special case: Only one div exists
			const firstDiv = divElements[0];
			const text = (firstDiv.textContent || '').trim();
			const hasOnlyBr = firstDiv.innerHTML.trim() === '<br>';
			const isEmpty = text === '' || text === '\u200B'; // Check for empty or zero-width space

			if (isEmpty || hasOnlyBr) {
				lineCount = 1; // If the single div is effectively empty, show only 1 line number
			} else {
				lineCount = 1; // If the single div has content, it's still 1 line
			}
		} else if (divCount > 1) {
			// Multiple divs, count them directly
			lineCount = divCount;
		} else {
			// No divs found (should ideally not happen with contenteditable if properly initialized)
			lineCount = 1; // Default to 1 line
		}

		// Clear existing line numbers
		lineNumbersContainer.innerHTML = '';

		// Create line number elements - ensure activeLineIndex is in bounds
		activeLineIndex = Math.max(0, Math.min(activeLineIndex, lineCount - 1));

		// Create all line number elements
		for (let i = 0; i < lineCount; i++) {
			const lineNumber = document.createElement('div');
			lineNumber.className = 'line-number';

			// Add active class
			if (i === activeLineIndex) {
				lineNumber.classList.add('active');
			}

			lineNumber.textContent = (i + 1).toString();
			lineNumbersContainer.appendChild(lineNumber);
		}
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

	function clearNormalModeBuffer() {
		normalModeBuffer = '';
		if (normalModeBufferTimeout) {
			clearTimeout(normalModeBufferTimeout);
			normalModeBufferTimeout = null;
		}
	}

	// Add helper function to get all text nodes
	function getAllTextNodes(node: Node): Text[] {
		const textNodes: Text[] = [];
		const walker = document.createTreeWalker(node, NodeFilter.SHOW_TEXT, null);

		let currentNode;
		while ((currentNode = walker.nextNode())) {
			textNodes.push(currentNode as Text);
		}
		return textNodes;
	}

	// New color picker variables
	let selectedColor = '#000000';
	let hexColor = '#000000';
	let rgbColor = { r: 0, g: 0, b: 0 };
	let hue = 0;
	let saturation = 100;
	let lightness = 50;
	let colorSelectionActive = false;

	const availableColors = [
		{ name: 'Black', value: '#000000' },
		{ name: 'Red', value: '#FF0000' },
		{ name: 'Green', value: '#008000' },
		{ name: 'Blue', value: '#0000FF' },
		{ name: 'Purple', value: '#800080' },
		{ name: 'Orange', value: '#FFA500' },
		{ name: 'Yellow', value: '#FFD700' }
	];

	// Color picker functions
	function updateColorFromHueOnly() {
		// Handle special cases for white and black at the extremes
		if (hue === 0) {
			// White at the beginning
			selectedColor = '#FFFFFF';
			hexColor = '#FFFFFF';
			rgbColor = { r: 255, g: 255, b: 255 };
		} else if (hue >= 355) {
			// Black at the end (wider range to ensure it can be selected)
			selectedColor = '#000000';
			hexColor = '#000000';
			rgbColor = { r: 0, g: 0, b: 0 };
		} else {
			// Regular color spectrum
			// Normalize the hue to 0-360 for the visible spectrum
			const normalizedHue = (hue * 340) / 360 + 10; // Skip the white and black at ends
			// Set default saturation and lightness for hue-only selection
			saturation = 100;
			lightness = 50;
			// Convert HSL to RGB
			const rgb = hslToRgb(normalizedHue, saturation / 100, lightness / 100);
			rgbColor = { r: rgb[0], g: rgb[1], b: rgb[2] };
			// Convert RGB to HEX
			hexColor = rgbToHex(rgbColor.r, rgbColor.g, rgbColor.b);
			selectedColor = hexColor;
		}
	}

	function updateColorFromHSL() {
		// Convert HSL to RGB
		const rgb = hslToRgb(hue, saturation / 100, lightness / 100);
		rgbColor = { r: rgb[0], g: rgb[1], b: rgb[2] };

		// Convert RGB to HEX
		hexColor = rgbToHex(rgbColor.r, rgbColor.g, rgbColor.b);
		selectedColor = hexColor;
	}

	function updateColorFromRGB() {
		// Convert RGB to HEX
		hexColor = rgbToHex(rgbColor.r, rgbColor.g, rgbColor.b);
		selectedColor = hexColor;

		// Convert RGB to HSL
		const hsl = rgbToHsl(rgbColor.r, rgbColor.g, rgbColor.b);
		hue = hsl[0];
		saturation = hsl[1] * 100;
		lightness = hsl[2] * 100;
	}

	function selectPresetColor(colorValue: string) {
		hexColor = colorValue;

		// Convert HEX to RGB
		const rgb = hexToRgb(hexColor);
		if (rgb) {
			rgbColor = { r: rgb[0], g: rgb[1], b: rgb[2] };

			// Convert RGB to HSL
			const hsl = rgbToHsl(rgbColor.r, rgbColor.g, rgbColor.b);
			hue = hsl[0];
			saturation = hsl[1] * 100;
			lightness = hsl[2] * 100;

			selectedColor = hexColor;
		}
	}

	// Color conversion helper functions
	function hexToRgb(hex: string): [number, number, number] | null {
		const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
		return result ? [parseInt(result[1], 16), parseInt(result[2], 16), parseInt(result[3], 16)] : null;
	}

	function rgbToHex(r: number, g: number, b: number): string {
		return (
			'#' +
			[r, g, b]
				.map((x) => {
					const hex = Math.max(0, Math.min(255, Math.round(x))).toString(16);
					return hex.length === 1 ? '0' + hex : hex;
				})
				.join('')
		);
	}

	function hslToRgb(h: number, s: number, l: number): [number, number, number] {
		h /= 360;
		let r, g, b;

		if (s === 0) {
			// Achromatic (gray)
			r = g = b = l;
		} else {
			const hue2rgb = (p: number, q: number, t: number): number => {
				if (t < 0) t += 1;
				if (t > 1) t -= 1;
				if (t < 1 / 6) return p + (q - p) * 6 * t;
				if (t < 1 / 2) return q;
				if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
				return p;
			};

			const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
			const p = 2 * l - q;

			r = hue2rgb(p, q, h + 1 / 3);
			g = hue2rgb(p, q, h);
			b = hue2rgb(p, q, h - 1 / 3);
		}

		return [Math.round(r * 255), Math.round(g * 255), Math.round(b * 255)];
	}

	function rgbToHsl(r: number, g: number, b: number): [number, number, number] {
		r /= 255;
		g /= 255;
		b /= 255;

		const max = Math.max(r, g, b);
		const min = Math.min(r, g, b);
		let h = 0;
		let s = 0;
		const l = (max + min) / 2;

		if (max !== min) {
			const d = max - min;
			s = l > 0.5 ? d / (2 - max - min) : d / (max + min);

			switch (max) {
				case r:
					h = (g - b) / d + (g < b ? 6 : 0);
					break;
				case g:
					h = (b - r) / d + 2;
					break;
				case b:
					h = (r - g) / d + 4;
					break;
			}

			h *= 60;
		}

		return [Math.round(h), s, l];
	}


	// Simple onMount function that ensures all content is visible immediately
	onMount(() => {
		console.log('Document page mounted, setting documentReady');
		// Set document ready immediately for content
		documentReady = true;
		
		// Use a simple timeout to delay the navbar appearance
		setTimeout(() => {
			console.log('Setting navbarReady to true');
			navbarReady = true;
			console.log('navbarReady is now:', navbarReady);
			
			// Log the navbar container opacity after a short delay
			setTimeout(() => {
				const navbarContainer = document.querySelector('.navbar-container');
				if (navbarContainer) {
					console.log('Navbar container style:', navbarContainer.getAttribute('style'));
					console.log('Navbar container opacity:', window.getComputedStyle(navbarContainer).opacity);
				}
			}, 100);
		}, 300);
	});
	
	// Define our command functions object with formatting commands
	const commandFunctions: CommandFunctions = {
		applyBoldFormatting: () => {
			console.debug('Executing bold formatting command (NORMAL mode)');
			if (!document.queryCommandSupported('bold')) {
				showCommandError('Bold formatting not supported');
				return;
			}

			const selection = window.getSelection();
			// ONLY apply if text is actually selected in NORMAL mode
			if (selection && !selection.isCollapsed) {
				document.execCommand('bold', false);
				// MutationObserver handles content updates
				showCommandError('Bold formatting applied');
			} else {
				console.debug('Bold command ignored: No text selected in NORMAL mode.');
			}
		},
		applyItalicFormatting: () => {
			console.debug('Executing italic formatting command (NORMAL mode)');
			if (!document.queryCommandSupported('italic')) {
				showCommandError('Italic formatting not supported');
				return;
			}

			const selection = window.getSelection();
			// ONLY apply if text is actually selected in NORMAL mode
			if (selection && !selection.isCollapsed) {
				document.execCommand('italic', false);
				// MutationObserver handles content updates
				showCommandError('Italic formatting applied');
			} else {
				console.debug('Italic command ignored: No text selected in NORMAL mode.');
			}
		},
		applyUnderlineFormatting: () => {
			console.debug('Executing underline formatting command (NORMAL mode)');
			if (!document.queryCommandSupported('underline')) {
				showCommandError('Underline formatting not supported');
				return;
			}

			const selection = window.getSelection();
			// ONLY apply if text is actually selected in NORMAL mode
			if (selection && !selection.isCollapsed) {
				document.execCommand('underline', false);
				// MutationObserver handles content updates

				// Fix potential nested font/u issues (keep this logic)
				if (editorElement) {
					const underlineElements = editorElement.querySelectorAll('u');
					for (const uElem of underlineElements) {
						const fontElements = uElem.querySelectorAll('font');
						if (fontElements.length > 1) {
							const fragment = document.createDocumentFragment();
							Array.from(fontElements).forEach(fontElem => {
								const color = fontElem.getAttribute('color');
								const content = fontElem.innerHTML;
								const newFont = document.createElement('font');
								if (color) newFont.setAttribute('color', color);
								const newU = document.createElement('u');
								newU.innerHTML = content;
								newFont.appendChild(newU);
								fragment.appendChild(newFont);
							});
							if (uElem.parentNode) {
								uElem.parentNode.replaceChild(fragment, uElem);
							}
						}
					}
				}
				showCommandError('Underline formatting applied');
			} else {
				console.debug('Underline command ignored: No text selected in NORMAL mode.');
			}
		},
		// Add document switching commands
		switchToDocument1: () => {
			console.debug('Switching to document 1');
			const index = 0;
			if (projectDocuments && projectDocuments[index]) {
				console.log(`Switching to document 1:`, projectDocuments[index]);
				switchDocument(projectDocuments[index].id);
			} else {
				console.log(`No document at index 1`);
				showCommandError(`No document 1 available`);
			}
		},
		switchToDocument2: () => {
			console.debug('Switching to document 2');
			const index = 1;
			if (projectDocuments && projectDocuments[index]) {
				console.log(`Switching to document 2:`, projectDocuments[index]);
				switchDocument(projectDocuments[index].id);
			} else {
				console.log(`No document at index 2`);
				showCommandError(`No document 2 available`);
			}
		},
		switchToDocument3: () => {
			console.debug('Switching to document 3');
			const index = 2;
			if (projectDocuments && projectDocuments[index]) {
				console.log(`Switching to document 3:`, projectDocuments[index]);
				switchDocument(projectDocuments[index].id);
			} else {
				console.log(`No document at index 3`);
				showCommandError(`No document 3 available`);
			}
		},
		switchToDocument4: () => {
			console.debug('Switching to document 4');
			const index = 3;
			if (projectDocuments && projectDocuments[index]) {
				console.log(`Switching to document 4:`, projectDocuments[index]);
				switchDocument(projectDocuments[index].id);
			} else {
				console.log(`No document at index 4`);
				showCommandError(`No document 4 available`);
			}
		},
		switchToDocument5: () => {
			console.debug('Switching to document 5');
			const index = 4;
			if (projectDocuments && projectDocuments[index]) {
				console.log(`Switching to document 5:`, projectDocuments[index]);
				switchDocument(projectDocuments[index].id);
			} else {
				console.log(`No document at index 5`);
				showCommandError(`No document 5 available`);
			}
		},
		switchToDocument6: () => {
			console.debug('Switching to document 6');
			const index = 5;
			if (projectDocuments && projectDocuments[index]) {
				console.log(`Switching to document 6:`, projectDocuments[index]);
				switchDocument(projectDocuments[index].id);
			} else {
				console.log(`No document at index 6`);
				showCommandError(`No document 6 available`);
			}
		},
		switchToDocument7: () => {
			console.debug('Switching to document 7');
			const index = 6;
			if (projectDocuments && projectDocuments[index]) {
				console.log(`Switching to document 7:`, projectDocuments[index]);
				switchDocument(projectDocuments[index].id);
			} else {
				console.log(`No document at index 7`);
				showCommandError(`No document 7 available`);
			}
		},
		switchToDocument8: () => {
			console.debug('Switching to document 8');
			const index = 7;
			if (projectDocuments && projectDocuments[index]) {
				console.log(`Switching to document 8:`, projectDocuments[index]);
				switchDocument(projectDocuments[index].id);
			} else {
				console.log(`No document at index 8`);
				showCommandError(`No document 8 available`);
			}
		},
		switchToDocument9: () => {
			console.debug('Switching to document 9');
			const index = 8;
			if (projectDocuments && projectDocuments[index]) {
				console.log(`Switching to document 9:`, projectDocuments[index]);
				switchDocument(projectDocuments[index].id);
			} else {
				console.log(`No document at index 9`);
				showCommandError(`No document 9 available`);
			}
		},
		openColorPicker: () => {
			console.debug('Opening color picker');
			// Reset state on open for consistency
			hue = 0; // Reset hue (e.g., to white)
			updateColorFromHueOnly(); // Update selectedColor based on reset hue
			showColorPicker = true;
		},
		enterInsertMode: () => {
			console.debug('Executing enter insert mode command');
			if (editorMode !== 'INSERT') {
				editorMode = 'INSERT';
				showCommandError('-- INSERT --');
				clearNormalModeBuffer();
			}
		},
		// Add the movement functions here
		moveLeft: () => {
			if (editorMode !== 'NORMAL') {
				console.debug('Ignoring moveLeft command: Not in NORMAL mode.');
				return; // Only allow in NORMAL mode
			}
			console.debug('Executing moveLeft command');
			moveLeft(); // Call the existing function
		},
		moveRight: () => {
			if (editorMode !== 'NORMAL') {
				console.debug('Ignoring moveRight command: Not in NORMAL mode.');
				return; // Only allow in NORMAL mode
			}
			console.debug('Executing moveRight command');
			moveRight(); // Call the existing function
		},
		moveUp: () => {
			if (editorMode !== 'NORMAL') {
				console.debug('Ignoring moveUp command: Not in NORMAL mode.');
				return; // Only allow in NORMAL mode
			}
			console.debug('Executing moveUp command');
			moveUp(); // Call the existing function
		},
		moveDown: () => {
			if (editorMode !== 'NORMAL') {
				console.debug('Ignoring moveDown command: Not in NORMAL mode.');
				return; // Only allow in NORMAL mode
			}
			console.debug('Executing moveDown command');
			moveDown(); // Call the existing function
		},
		moveToStartOfLine: () => { // Add this function
			if (editorMode !== 'NORMAL') {
				console.debug('Ignoring moveToStartOfLine command: Not in NORMAL mode.');
				return; // Only allow in NORMAL mode
			}
			console.debug('Executing moveToStartOfLine command');
			moveToStartOfLine(); // Call the existing function
		},
		moveToEndOfLine: () => {
			if (editorMode !== 'NORMAL') {
				console.debug('Ignoring moveToEndOfLine command: Not in NORMAL mode.');
				return; // Only allow in NORMAL mode
			}
			console.debug('Executing moveToEndOfLine command');
			moveToEndOfLine(); // Call the existing function
		},
		moveToEndOfDocument: () => {
			if (editorMode !== 'NORMAL') {
				console.debug('Ignoring moveToEndOfDocument command: Not in NORMAL mode.');
				return; // Only allow in NORMAL mode
			}
			console.debug('Executing moveToEndOfDocument command');
			moveToEndOfDocument(); // Call the existing function
		},
		moveToStartOfDocument: () => {
			if (editorMode !== 'NORMAL') {
				console.debug('Ignoring moveToStartOfDocument command: Not in NORMAL mode.');
				return; // Only allow in NORMAL mode
			}
			console.debug('Executing moveToStartOfDocument command');
			moveToStartOfDocument(); // Call the existing function
		},
		toggleCommandSheet: () => {
			// No mode check needed for this
			console.debug('Executing toggleCommandSheet command');
			showCommands = !showCommands;
		},
		findNextMatch: () => { 
			if (editorMode !== 'NORMAL') {
				console.debug('Ignoring findNextMatch command: Not in NORMAL mode.');
				return; // Only allow in NORMAL mode
			}
			console.debug('Executing findNextMatch command');
			findNextMatch(false); // false for forward
		},
		findPreviousMatch: () => {
			if (editorMode !== 'NORMAL') {
				console.debug('Ignoring findPreviousMatch command: Not in NORMAL mode.');
				return; // Only allow in NORMAL mode
			}
			console.debug('Executing findPreviousMatch command');
			findNextMatch(true); // true for reverse
		},
		deleteSelectedText: () => { 
			if (editorMode !== 'NORMAL') {
				console.debug('Ignoring deleteSelectedText command: Not in NORMAL mode.');
				return; // Only allow in NORMAL mode
			}
			console.debug('Executing deleteSelectedText command');
			clearNormalModeBuffer(); // Clear any pending sequence
			deleteHighlightedText(); // Call the existing helper function
		},
		yankText: () => { 
			if (editorMode !== 'NORMAL') {
				console.debug('Ignoring yankText command: Not in NORMAL mode.');
				return; // Only allow in NORMAL mode
			}
			console.debug('Executing yankText command (calling copyText)');
			clearNormalModeBuffer();
			copyText(); // Call the existing copy/yank helper
		},
		deleteCurrentLine: () => {
			if (editorMode !== 'NORMAL') {
				console.debug('Ignoring deleteCurrentLine command: Not in NORMAL mode.');
				return; // Only allow in NORMAL mode
			}
			console.debug('Executing deleteCurrentLine command');
			clearNormalModeBuffer(); // Clear buffer
			deleteCurrentLine(); // Call the existing helper function
		},
		pasteText: async () => {
			if (editorMode !== 'NORMAL') {
				console.debug('Ignoring pasteText command: Not in NORMAL mode.');
				return; // Only allow in NORMAL mode
			}
			console.debug('Executing pasteText command');
			clearNormalModeBuffer();
			try {
				const clipboardText = await navigator.clipboard.readText();
				if (!clipboardText || !editorElement) return;
				
				// Similar logic to handlePaste, but using read text
				const selection = window.getSelection();
				if (!selection || !selection.rangeCount) return;
				const range = selection.getRangeAt(0);
				const startOffsetInDocument = getTextOffset(range.startContainer, range.startOffset);
				
				if (!range.collapsed) {
					range.deleteContents(); // Delete selected text if any
				}

				let currentContent = getEditorContent();
				const contentBefore = currentContent.substring(0, startOffsetInDocument);
				const contentAfter = currentContent.substring(startOffsetInDocument);
				const normalizedClipboardText = clipboardText.replace(/\r\n/g, '\n'); // Normalize newlines
				const newContent = contentBefore + normalizedClipboardText + contentAfter;
				
				const insertedLength = normalizedClipboardText.length;
				const targetOffset = startOffsetInDocument + insertedLength;

				safelySetEditorContent(newContent);
				
				// Restore cursor position
				setTimeout(() => {
					console.log(`Restoring cursor position after programmatic paste to offset: ${targetOffset}`);
					setCursorPositionByOffset(targetOffset);
					updateLineNumbers();
					ensureCursorVisible();
					adjustEditorHeight();
				}, 0); 

			} catch (err) {
				console.error('Failed to read clipboard contents: ', err);
				showCommandError('Failed to paste: Check clipboard permissions');
			}
		},
		toggleChatAssistant: () => {
			isChatOpen = !isChatOpen;
			console.log(`Toggling chat visibility to: ${isChatOpen}`);
		},
	};

	// Global keyboard event handler for keybindings
	function handleKeybindingKeyDown(event: KeyboardEvent) {
		// If chat input is focused, don't process editor keybindings
		if (document.activeElement === chatInputElementRef) {
			return;
		}

		// Only process keybindings if we are in NORMAL mode
		if (editorMode !== 'NORMAL') {
			// Allow the key event to proceed for other handlers (like the editor's own keydown)
			return; 
		}
		// Block keybindings if color picker is open
		if (showColorPicker) {
			return;
		}

		console.debug('Keyboard event received:', {
			key: event.key,
			altDown: event.altKey,
			ctrlDown: event.ctrlKey,
			shiftDown: event.shiftKey
		});

		// Convert event to our input format for debugging
		const input = keybindingMap.keyEventToInput(event);
		const mapKey = keybindingMap.getMapKey(input);
		console.debug('Converted to map key:', mapKey);

		// Check active bindings for debugging
		console.debug('Current active bindings:', keybindings.activeBindings);

		// Try to handle the input
		const wasHandled = keybindingMap.handleKeyboardInput(event, commandFunctions);
		console.debug('Keyboard input was handled:', wasHandled);

		if (wasHandled) {
			console.debug('Command was executed successfully');
		} else {
			console.debug('No matching keybinding found for:', mapKey);
		}
	}

	// Handle color picker keyboard events
	function handleColorPickerKeyDown(event: KeyboardEvent) {
		if (!showColorPicker) return;

		// Get dynamic movement keys (lowercase for comparison)
		const leftKey = keybindings.activeBindings.moveLeft?.kd?.toLowerCase();
		const rightKey = keybindings.activeBindings.moveRight?.kd?.toLowerCase();
		const upKey = keybindings.activeBindings.moveUp?.kd?.toLowerCase();
		const downKey = keybindings.activeBindings.moveDown?.kd?.toLowerCase();

		const eventKeyLower = event.key.toLowerCase();
		let handled = false;

		switch(eventKeyLower) { // Switch on lowercase key
			case 'escape':
				event.preventDefault();
				showColorPicker = false;
				handled = true;
				break;
			case 'enter':
				event.preventDefault();
				if (selectedColor) {
					applyTextColor(selectedColor);
						showColorPicker = false;
				}
				handled = true;
				break;
			// Keep hardcoded arrow keys
			case 'arrowleft':
				event.preventDefault();
				hue = (hue - 5 + 365) % 365; 
				updateColorFromHueOnly();
				handled = true;
				break; 
			case 'arrowright':
				event.preventDefault();
				hue = (hue + 5) % 365;
				updateColorFromHueOnly();
				handled = true;
				break;
			case 'arrowup':
				event.preventDefault();
				hue = (hue + 15) % 365; 
				updateColorFromHueOnly();
				handled = true;
				break;
			case 'arrowdown':
				event.preventDefault();
				hue = (hue - 15 + 365) % 365;
				updateColorFromHueOnly();
				handled = true;
				break;
			// Handle dynamic keys if not an arrow/escape/enter
			default:
				if (leftKey && eventKeyLower === leftKey) {
					event.preventDefault();
					hue = (hue - 5 + 365) % 365;
					updateColorFromHueOnly();
					handled = true;
				} else if (rightKey && eventKeyLower === rightKey) {
					event.preventDefault();
					hue = (hue + 5) % 365;
					updateColorFromHueOnly();
					handled = true;
				} else if (upKey && eventKeyLower === upKey) {
					event.preventDefault();
					hue = (hue + 15) % 365;
					updateColorFromHueOnly();
					handled = true;
				} else if (downKey && eventKeyLower === downKey) {
					event.preventDefault();
					hue = (hue - 15 + 365) % 365;
					updateColorFromHueOnly();
					handled = true;
				} 
		}

		// If the key wasn't handled by the switch or the dynamic checks, prevent default anyway
		if (!handled) {
			event.preventDefault(); 
		}
	}

	onMount(() => {
		console.debug('Component mounted, initializing keybindings');
		
		// Initialize keybindings
		keybindings.fetchAndUpdateBindings()
			.then(() => {
				console.debug('Keybindings initialized:', keybindings.activeBindings);
				window.addEventListener('keydown', handleKeybindingKeyDown);
			})
			.catch((error) => {
				console.error('Error initializing keybindings:', error);
			});

		// Return cleanup function
		return () => {
			console.debug('Cleaning up keyboard event listener');
			window.removeEventListener('keydown', handleKeybindingKeyDown);
		};
	});

	// Movement functions
	function moveLeft() {
		if (!editorElement) return;
		const selection = window.getSelection();
		if (!selection || !selection.rangeCount) return;

		const range = selection.getRangeAt(0);
		const allDivs = Array.from(editorElement.querySelectorAll('div'));
		
		// If selection is not collapsed, move to the start of the selection first
		if (!range.collapsed) {
			range.collapse(true); // Collapse to the start
			selection.removeAllRanges();
			selection.addRange(range);
			updateCursorPosition(); // Update state after collapsing
			updateLineNumbers();
			ensureCursorVisible();
			return; // Let next key press handle actual movement
		}
		
		const currentContainer = range.startContainer;
		const currentOffset = range.startOffset;
		const newRange = document.createRange();
		let successfullyMoved = false;

		if (currentOffset > 0) {
			// Try moving one position back in the current node
			try {
				// If it's a text node, move within text
				if (currentContainer.nodeType === Node.TEXT_NODE) {
					newRange.setStart(currentContainer, currentOffset - 1);
				} else {
					// If it's an element node, select the node before the current offset
					// This handles moving out of elements like <font> or <u> correctly
					const nodeBefore = currentContainer.childNodes[currentOffset - 1];
					newRange.setStartAfter(nodeBefore); // Position cursor after the previous node
				}
				successfullyMoved = true;
			} catch (e) {
				console.error('Error setting range in moveLeft (current node):', e, {container: currentContainer, offset: currentOffset});
				return;
			}
		} else {
			// At the start of the current container node
			// Find the visually previous node we can move into
			let nodeToMoveTo: Node | null = null;
			let offsetInNode = 0;
			let currentNode: Node | null = currentContainer;

			// Traverse siblings first
			while (currentNode && currentNode !== editorElement && currentNode.parentNode !== editorElement ) { 
				if (currentNode.previousSibling) {
					nodeToMoveTo = currentNode.previousSibling;
					// Find the deepest last child that is a text node or an element node
					while (nodeToMoveTo && (nodeToMoveTo.nodeType === Node.ELEMENT_NODE || nodeToMoveTo.nodeType === Node.TEXT_NODE) && nodeToMoveTo.lastChild) {
						nodeToMoveTo = nodeToMoveTo.lastChild;
					}
					// Set position at the end of the found node
					if (nodeToMoveTo && nodeToMoveTo.nodeType === Node.TEXT_NODE) {
						offsetInNode = nodeToMoveTo.textContent?.length || 0;
					} else if (nodeToMoveTo) { 
						offsetInNode = nodeToMoveTo.childNodes.length;
					} else { // Fallback if sibling is weird (comment etc.)
						nodeToMoveTo = currentNode.previousSibling; 
						offsetInNode = 0; 
					}
					break;
				} else {
					// No previous sibling, move up to parent
					currentNode = currentNode.parentNode;
				}
			}

			// If we didn't find a suitable sibling, or we reached the line div, move to the previous line
			if (!nodeToMoveTo && activeLineIndex > 0) {
				const prevDiv = allDivs[activeLineIndex - 1];
				const textNodes = getAllTextNodes(prevDiv);

				if (textNodes.length > 0) {
					// If previous line has text, target the end of the last text node
					nodeToMoveTo = textNodes[textNodes.length - 1];
					offsetInNode = nodeToMoveTo.textContent?.length || 0;
				} else {
					// Previous line is empty or has only non-text nodes (<br>), target start of the div
					nodeToMoveTo = prevDiv; 
					offsetInNode = 0; 
				}
				
				activeLineIndex--; // Update line index
				// nodeToMoveTo and offsetInNode are now set
			} else if (!nodeToMoveTo) {
				// At the very beginning of the document or couldn't find sibling
				return;
			}

			// Set the range based on the found node and offset
			if (nodeToMoveTo) {
				try {
					const maxOffset = nodeToMoveTo.nodeType === Node.TEXT_NODE ? (nodeToMoveTo.textContent?.length || 0) : nodeToMoveTo.childNodes.length;
					// Ensure offset is not negative and not exceeding maxOffset
					const safeOffset = Math.max(0, Math.min(offsetInNode, maxOffset)); 
					newRange.setStart(nodeToMoveTo, safeOffset);
					successfullyMoved = true;
				} catch (e) {
					console.error('Error setting range in moveLeft (previous node/line):', e, { node: nodeToMoveTo, offset: offsetInNode });
					// Attempt fallback: set to start of the node
					try {
						newRange.setStart(nodeToMoveTo, 0);
						successfullyMoved = true;
					} catch (fallbackError) {
						console.error('Fallback setting range failed in moveLeft:', fallbackError);
						return;
					}
				}
			}
		}

		if (successfullyMoved) {
			newRange.collapse(true);
			selection.removeAllRanges();
			selection.addRange(newRange);

			updateCursorPosition();
			updateLineNumbers();
			ensureCursorVisible();
		} else {
			console.warn('MoveLeft command executed but no movement occurred.');
		}
	}

	function moveRight() {
		if (!editorElement) return;
		const selection = window.getSelection();
		if (!selection || !selection.rangeCount) return;

		const range = selection.getRangeAt(0);
		const allDivs = Array.from(editorElement.querySelectorAll('div'));

		// If selection is not collapsed, move to the end of the selection first
		if (!range.collapsed) {
			range.collapse(false); // Collapse to the end
			selection.removeAllRanges();
			selection.addRange(range);
			updateCursorPosition(); // Update state after collapsing
			updateLineNumbers();
			ensureCursorVisible();
			return; // Let next key press handle actual movement
		}

		const currentContainer = range.startContainer;
		const currentOffset = range.startOffset;
		const newRange = document.createRange();
		let successfullyMoved = false;
		const currentDiv = allDivs[activeLineIndex];
		const lineIsEmpty = !currentDiv?.textContent || currentDiv.textContent.trim() === '' || currentDiv.textContent.trim() === '\u200B' || (currentDiv.childNodes.length === 1 && currentDiv.firstChild?.nodeName === 'BR');

		if (lineIsEmpty && activeLineIndex < allDivs.length - 1) {
			console.log('Moving right from empty line, calling moveDown()');
			moveDown();
			return;
		}

		// Check if we can move forward within the current node
		let canMoveInNode = false;
		if (currentContainer.nodeType === Node.TEXT_NODE) {
			canMoveInNode = currentOffset < (currentContainer.textContent?.length || 0);
		} else { // Element node
			canMoveInNode = currentOffset < currentContainer.childNodes.length;
		}

		if (canMoveInNode) {
			// Move one position forward in the current node
			try {
				if (currentContainer.nodeType === Node.TEXT_NODE) {
					newRange.setStart(currentContainer, currentOffset + 1);
				} else {
					// If element node, select the node *at* the current offset
					const nodeAfter = currentContainer.childNodes[currentOffset];
					newRange.setStartBefore(nodeAfter);
				}
				successfullyMoved = true;
			} catch (e) {
				console.error('Error setting range in moveRight (current node):', e, {container: currentContainer, offset: currentOffset});
				return;
			}
		} else {
			// At the end of the current container node
			// Find the visually next node we can move into within the same line (div)
			let nodeToMoveTo: Node | null = null;
			let offsetInNode = 0;
			let currentNode: Node | null = currentContainer;
			let parentDiv: Node | null = currentNode;
			while(parentDiv && parentDiv.parentNode && parentDiv.parentNode !== editorElement) {
				parentDiv = parentDiv.parentNode;
			}

			while (currentNode && currentNode !== editorElement && currentNode !== parentDiv) { // Stop if we reach the editor or the line's div
				if (currentNode.nextSibling) {
					nodeToMoveTo = currentNode.nextSibling;
					// Find the deepest first child that is a text node or element node
					while (nodeToMoveTo && (nodeToMoveTo.nodeType === Node.ELEMENT_NODE || nodeToMoveTo.nodeType === Node.TEXT_NODE) && nodeToMoveTo.firstChild) {
						nodeToMoveTo = nodeToMoveTo.firstChild;
					}
					offsetInNode = 0; // Start at the beginning of the next node
					break;
				} else {
					// No next sibling, move up to parent
					currentNode = currentNode.parentNode;
				}
			}

			// If we found a next node within the same line
			if (nodeToMoveTo) {
				try {
					const maxOffset = nodeToMoveTo.nodeType === Node.TEXT_NODE ? 0 : nodeToMoveTo.childNodes.length;
					const safeOffset = Math.max(0, Math.min(offsetInNode, maxOffset));
					newRange.setStart(nodeToMoveTo, safeOffset);
					successfullyMoved = true;
				} catch (e) {
					console.error('Error setting range in moveRight (next node):', e, { node: nodeToMoveTo, offset: offsetInNode });
					return;
				}
			} else {
				// At the end of the line, attempt to move down if possible
				if (activeLineIndex < allDivs.length - 1) {
					console.log('Reached end of line, attempting moveDown()');
					moveDown(); // Call moveDown to handle moving to the next line
					return; // moveDown handles updates, so we return here
				} else {
					// At the very end of the document
					return;
				}
			}
		}

		if (successfullyMoved) {
			newRange.collapse(true);
			selection.removeAllRanges();
			selection.addRange(newRange);
			updateCursorPosition();
			updateLineNumbers();
			ensureCursorVisible();
		} else if (!successfullyMoved && activeLineIndex < allDivs.length - 1) {
			// Fallback logging
			console.warn('MoveRight reached end of line but moveDown was not called?');
		}
	}

	function moveUp() {
		if (!editorElement) return;
		const selection = window.getSelection();
		if (!selection || !selection.rangeCount) return;

		if (activeLineIndex > 0) {
			const allDivs = Array.from(editorElement.querySelectorAll('div'));
			const targetDiv = allDivs[activeLineIndex - 1];
			if (targetDiv) {
				const range = document.createRange();
				// Check if empty or just zero-width space
				const lineIsEmpty = !targetDiv.textContent || targetDiv.textContent.trim() === '\u200B'; 

				if (lineIsEmpty) {
					// If target line is empty, place cursor at its start
					range.setStart(targetDiv, 0);
				} else {
					// Line has content, try to maintain column
					const targetColumn = Math.min(cursorColumn - 1, targetDiv.textContent?.length || 0);
					if (targetDiv.firstChild && targetDiv.firstChild.nodeType === Node.TEXT_NODE) {
						range.setStart(targetDiv.firstChild, targetColumn);
					} else {
						// Fallback if first child isn't text (or line has complex nodes)
						// We need a more robust way to find the node at the target column
						let accumulatedOffset = 0;
						let targetNode: Node | null = null;
						let targetOffset = 0;
						const walker = document.createTreeWalker(targetDiv, NodeFilter.SHOW_TEXT);
						let currentNode: Node | null;
						while ((currentNode = walker.nextNode())) {
							const nodeLength = currentNode.textContent?.length || 0;
							if (accumulatedOffset + nodeLength >= targetColumn) {
								targetNode = currentNode;
								targetOffset = targetColumn - accumulatedOffset;
								break;
							}
							accumulatedOffset += nodeLength;
						}
						// If still no target node (e.g., column beyond actual text), use last text node
						if (!targetNode) {
							const allTextNodes = getAllTextNodes(targetDiv);
							if (allTextNodes.length > 0) {
								targetNode = allTextNodes[allTextNodes.length - 1];
								targetOffset = targetNode.textContent?.length || 0;
							} else {
								// Fallback: start of the div if truly no text nodes
								targetNode = targetDiv;
								targetOffset = 0;
							}
						}
						// Set the range
						try {
							range.setStart(targetNode, targetOffset);
						} catch(e) {
							console.error("Error setting range in moveUp fallback:", e, {targetNode, targetOffset});
							range.setStart(targetDiv, 0); // Ultimate fallback
						}
					}
				}

				range.collapse(true);
				selection.removeAllRanges();
				selection.addRange(range);
				activeLineIndex--;
				updateCursorPosition();
				updateLineNumbers();
				ensureCursorVisible();
			}
		}
	}

	function moveDown() {
		if (!editorElement) return;
		const selection = window.getSelection();
		if (!selection || !selection.rangeCount) return;

		const allDivs = Array.from(editorElement.querySelectorAll('div'));
		if (activeLineIndex < allDivs.length - 1) {
			const targetDiv = allDivs[activeLineIndex + 1];
			if (targetDiv) {
				const range = document.createRange();
				// Check if empty or just zero-width space
				const lineIsEmpty = !targetDiv.textContent || targetDiv.textContent.trim() === '\u200B'; 

				if (lineIsEmpty) {
					// If target line is empty, place cursor at its start
					range.setStart(targetDiv, 0);
				} else {
					// Line has content, try to maintain column
					const targetColumn = Math.min(cursorColumn - 1, targetDiv.textContent?.length || 0);
					if (targetDiv.firstChild && targetDiv.firstChild.nodeType === Node.TEXT_NODE) {
						range.setStart(targetDiv.firstChild, targetColumn);
					} else {
						// Fallback if first child isn't text (or line has complex nodes)
						let accumulatedOffset = 0;
						let targetNode: Node | null = null;
						let targetOffset = 0;
						const walker = document.createTreeWalker(targetDiv, NodeFilter.SHOW_TEXT);
						let currentNode: Node | null;
						while ((currentNode = walker.nextNode())) {
							const nodeLength = currentNode.textContent?.length || 0;
							if (accumulatedOffset + nodeLength >= targetColumn) {
								targetNode = currentNode;
								targetOffset = targetColumn - accumulatedOffset;
								break;
							}
							accumulatedOffset += nodeLength;
						}
						// If still no target node, use last text node
						if (!targetNode) {
							const allTextNodes = getAllTextNodes(targetDiv);
							if (allTextNodes.length > 0) {
								targetNode = allTextNodes[allTextNodes.length - 1];
								targetOffset = targetNode.textContent?.length || 0;
							} else {
								// Fallback: start of the div if truly no text nodes
								targetNode = targetDiv;
								targetOffset = 0;
							}
						}
						// Set the range
						try {
							range.setStart(targetNode, targetOffset);
						} catch(e) {
							console.error("Error setting range in moveDown fallback:", e, {targetNode, targetOffset});
							range.setStart(targetDiv, 0); // Ultimate fallback
						}
					}
				}

				range.collapse(true);
				selection.removeAllRanges();
				selection.addRange(range);
				activeLineIndex++;
				updateCursorPosition();
				updateLineNumbers();
				ensureCursorVisible();
			}
		}
	}

	function handleInsertModeKeyDown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			exitInsertMode(); // Assuming exitInsertMode is defined in the outer scope
			event.preventDefault();
			return;
		}

		// --- This is the logic that should be present ---

		// Explicitly check for formatting shortcuts in INSERT mode
		const eventInput = {
			// Use kd and ensure lowercase for comparison
			kd: event.key.toLowerCase(), 
			altDown: event.altKey,
			ctrlDown: event.ctrlKey,
			shiftDown: event.shiftKey
		};

		const bindings = keybindings.activeBindings;

		// *** Helper function defined INSIDE handleInsertModeKeyDown ***
		const checkBinding = (binding: KeyboardInput | undefined): boolean => {
			if (!binding) return false;
			// Compare using kd
			return (
				binding.kd.toLowerCase() === eventInput.kd && // Compare lowercase key
				binding.altDown === eventInput.altDown &&
					binding.ctrlDown === eventInput.ctrlDown &&
					binding.shiftDown === eventInput.shiftDown
			);
		};
		// *** End of checkBinding definition ***

		// Now use the helper function
		if (bindings.bold && checkBinding(bindings.bold)) {
			console.log('Applying bold formatting in INSERT mode');
			commandFunctions.applyBoldFormatting?.();
			event.preventDefault();
			return;
		}

		if (bindings.italic && checkBinding(bindings.italic)) {
			console.log('Applying italic formatting in INSERT mode');
			commandFunctions.applyItalicFormatting?.();
			event.preventDefault();
			return;
		}

		if (bindings.underline && checkBinding(bindings.underline)) {
			console.log('Applying underline formatting in INSERT mode');
			commandFunctions.applyUnderlineFormatting?.();
			event.preventDefault();
			return;
		}
		if (bindings.openColorPicker && checkBinding(bindings.openColorPicker)) {
			console.log('Opening color picker in INSERT mode');
			commandFunctions.openColorPicker?.();
			event.preventDefault();
			return;
		}
		if (bindings.switchToDocument1 && checkBinding(bindings.switchToDocument1)) {
			console.log('Switching to document 1 in INSERT mode');
			commandFunctions.switchToDocument1?.();
			event.preventDefault();
			return;
		}
		if (bindings.switchToDocument2 && checkBinding(bindings.switchToDocument2)) {
			console.log('Switching to document 2 in INSERT mode');
			commandFunctions.switchToDocument2?.();
			event.preventDefault();
			return;
		}
		if (bindings.switchToDocument3 && checkBinding(bindings.switchToDocument3)) {
			console.log('Switching to document 3 in INSERT mode');
			commandFunctions.switchToDocument3?.();
			event.preventDefault();
			return;
		}
		if (bindings.switchToDocument4 && checkBinding(bindings.switchToDocument4)) {
			console.log('Switching to document 4 in INSERT mode');
			commandFunctions.switchToDocument4?.();
			event.preventDefault();
			return;
		}
		if (bindings.switchToDocument5 && checkBinding(bindings.switchToDocument5)) {
			console.log('Switching to document 5 in INSERT mode');
			commandFunctions.switchToDocument5?.();
			event.preventDefault();
			return;
		}
		if (bindings.switchToDocument6 && checkBinding(bindings.switchToDocument6)) {
			console.log('Switching to document 6 in INSERT mode');
			commandFunctions.switchToDocument6?.();
			event.preventDefault();
			return;
		}
		if (bindings.switchToDocument7 && checkBinding(bindings.switchToDocument7)) {
			console.log('Switching to document 7 in INSERT mode');
			commandFunctions.switchToDocument7?.();
			event.preventDefault();
			return;
		}
		if (bindings.switchToDocument8 && checkBinding(bindings.switchToDocument8)) {
			console.log('Switching to document 8 in INSERT mode');
			commandFunctions.switchToDocument8?.();
			event.preventDefault();
			return;
		}
		if (bindings.switchToDocument9 && checkBinding(bindings.switchToDocument9)) {
			console.log('Switching to document 9 in INSERT mode');
			commandFunctions.switchToDocument9?.();
			event.preventDefault();
			return;
		} 
		if (bindings.toggleCommandSheet && checkBinding(bindings.toggleCommandSheet)) {
			console.log('Toggling command sheet in INSERT mode');
			commandFunctions.toggleCommandSheet?.();
			event.preventDefault();
			return;
		}
	}
</script>

<svelte:head>
	<title>{documentData ? documentData.name : 'Document'} | Vynn</title>
</svelte:head>

{#each toasts as toast, i}
	<Toast message={toast.message} type={toast.type} onClose={() => removeToast(i)} />
{/each}


<div class="editor-page">
	<div class="background-image" style="background-image: url({backgroundImage})"></div>

	<!-- Minimal Navbar with fade-in animation -->
	<div class="navbar-container" class:fade-in-first={navbarReady} class:navbar-ready={navbarReady} style="opacity: {navbarReady ? 1 : 0}; transition: opacity 0.6s ease-out;">
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
						on:error={() => (userProfileImage = profileDefault)}
					/>
				</button>
				<ul class="dropdown-menu dropdown-menu-end dropdown-menu-dark profile-dropdown">
					<li>
						<button class="dropdown-item" on:click={goToAccount}>
							<i class="bi bi-person me-2"></i> My Account
						</button>
					</li>
					<li><hr class="dropdown-divider" /></li>
					<li>
						<button class="dropdown-item text-danger" on:click={handleLogout}>
							<i class="bi bi-box-arrow-right me-2"></i> Sign Out
						</button>
					</li>
				</ul>
			</div>
		</nav>
	</div>

	<!-- Project Document Switcher with fade-in animation -->
	{#if projectDocumentsLoaded}
		<div class="document-switcher" class:fade-in-second={navbarReady}>
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
				<button class="doc-button active" disabled aria-label="Document 1"> 1 </button>
	{/if}
		</div>
	{/if}

	<!-- Editor Container with animation -->
	<div class="editor-container" class:fade-in-third={documentReady}>
		{#if loading}
			<div class="loading"></div>
		{:else if error}
			<div class="error">Error loading document</div>
		{:else}
			<!-- Previous document (for animation) -->
			{#if isAnimating && previousDocumentContent}
				<div class="editor-wrapper previous {slideDirection}-exit">
					<div class="editor-content">
						<div class="line-numbers">
							{#each previousDocumentLines as line, i}
								<div class="line-number {i === previousActiveLineIndex ? 'active' : ''}">{i + 1}</div>
							{/each}
						</div>
						<div class="editor-contenteditable">{@html previousDocumentContent}</div>
					</div>
				</div>
			{/if}

			<!-- Current document -->
			<div class="editor-wrapper current {isAnimating ? `${slideDirection}-enter` : ''}">
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

	<!-- Fixed Status Bar with animation -->
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
			<button
				class="commands-toggle"
				on:click={() => (showCommands = !showCommands)}
				title="Toggle Commands Reference"
				aria-label="Toggle commands reference"
			>
				<i class="bi bi-info-circle"></i>
			</button>
			<span>Line: {cursorLine}, Col: {cursorColumn}</span>
		</div>
	</div>

	<!-- Add commands cheat sheet overlay -->
	<div class="commands-overlay" class:show-commands={showCommands}>
		<div class="commands-header">
				<h5>Command Reference</h5>
			<button class="commands-close" on:click={() => (showCommands = false)} aria-label="Close commands reference"
				>×</button
			>
		</div>
		<div class="commands-body">
			<!-- Restored original structure with dynamic keys -->
			<div class="commands-section">
				<h6>Mode Switching</h6>
				<ul>
					<li><span class="key">{insertModeKey}</span> Enter Insert mode</li>
					<li><span class="key">Esc</span> Return to Normal mode</li>
					<li><span class="key">:</span> Enter Command mode</li>
				</ul>
			</div>
			
			<div class="commands-section">
				<h6>Navigation</h6>
				<ul>
					<li><span class="key">{moveLeftKey}</span> Move left</li>
					<li><span class="key">{moveDownKey}</span> Move down</li>
					<li><span class="key">{moveUpKey}</span> Move up</li>
					<li><span class="key">{moveRightKey}</span> Move right</li>
					<li><span class="key">{startOfLineKey}</span> Start of line</li>
					<li><span class="key">{endOfLineKey}</span> End of line</li>
					<li><span class="key">{startOfDocKey}</span> Start of document (gg)</li>
					<li><span class="key">{endOfDocKey}</span> End of document</li>
				</ul>
			</div>
			
			<div class="commands-section">
				<h6>Editing</h6>
				<ul>
					<li><span class="key">{deleteSelectedKey}</span> Delete selected</li>
					<li><span class="key">{deleteLineKey}</span> Delete current line</li>
					<li><span class="key">{yankKey}</span> Copy (Yank)</li>
					<li><span class="key">{pasteKey}</span> Paste</li>
				</ul>
			</div>

			<div class="commands-section">
				<h6>Styling</h6>
				<ul>
					<li><span class="key">{boldKey}</span> Toggle Bold</li>
					<li><span class="key">{italicKey}</span> Toggle Italic</li>
					<li><span class="key">{underlineKey}</span> Toggle Underline</li>
					<li><span class="key">{colorPickerKey}</span> Open Color Picker</li>
				</ul>
			</div>
			
			<div class="commands-section">
				<h6>Search</h6>
				<ul>
					<li><span class="key">/</span> Search forward</li>
					<li><span class="key">?</span> Search backward</li>
					<li><span class="key">{nextMatchKey}</span> Next match</li>
					<li><span class="key">{prevMatchKey}</span> Previous match</li>
				</ul>
			</div>

			<div class="commands-section">
				<h6>Colon Commands</h6>
				<ul>
					<li><span class="key">:q</span> Quit document</li>
					<li><span class="key">:w</span> Save document</li>
					<li><span class="key">:wq</span> Save and quit</li>
					<li><span class="key">:export</span> Export to PDF</li>
					<li><span class="key">:%s/old/new/gi</span> Replace all</li>
				</ul>
			</div>

			<div class="commands-section">
				<h6>Document Switching</h6>
				<ul>
					<li><span class="key">{switchDoc1Key}</span> Switch to Document 1</li>
					<li><span class="key">{switchDoc2Key}</span> Switch to Document 2</li>
					<li><span class="key">{switchDoc3Key}</span> Switch to Document 3</li>
					<li><span class="key">{switchDoc4Key}</span> Switch to Document 4</li>
					<li><span class="key">{switchDoc5Key}</span> Switch to Document 5</li>
					<li><span class="key">{switchDoc6Key}</span> Switch to Document 6</li>
					<li><span class="key">{switchDoc7Key}</span> Switch to Document 7</li>
					<li><span class="key">{switchDoc8Key}</span> Switch to Document 8</li>
					<li><span class="key">{switchDoc9Key}</span> Switch to Document 9</li>
				</ul>
			</div>

			<div class="commands-section">
				<h6>Editor</h6>
				<ul>
					<li><span class="key">{toggleSheetKey}</span> Toggle Command Sheet</li>
				</ul>
			</div>
		</div>
	</div>

	{#if showColorPicker}
		<div class="color-picker" style="position: fixed; left: 50%; top: 50%; transform: translate(-50%, -50%);">
			<div class="hue-slider-container">
				<input
					type="range"
					min="0"
					max="365"
					bind:value={hue}
					class="hue-slider"
					on:input={updateColorFromHueOnly}
					tabindex="-1"
				/>
				<div class="color-slider-indicator" style="background-color: {selectedColor}"></div>
			</div>
		</div>
	{/if}

	{#if isChatOpen}
		<ChatAssistant
			bind:this={chatAssistantComponent}
			documentId={parseInt(documentId)}
			bind:isOpen={isChatOpen}
			on:close={() => (isChatOpen = false)}
			bind:messageInput={chatInputElementRef}
		/>
	{/if}
</div>