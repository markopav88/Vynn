/*
/ editor-commands.ts
/
/ File containing functions and logic required for vimm commands and functions
/
/ Summary:
/ EditorCommand: Structure for a given command
/ movementCommands: Const Mapping containing various movement functions
/ normalModeKeybindings: Const Mapping containing all valid NORMAL mode commands
/ documentCommands: Const Mapping containing all movement commands between documents
/ ctrlKeybindings: Const Mapper of Cntrl Bindings
/ getCursorRect: Function to get cursor rectangle
/ switchToDocumentByIndex: Helper Function to switch to a document by index
/ handleNormalModeKeydown: Function to handle NORMAL key press
/ executeCommand: Function to execute a command
*/

// Define command types
export type EditorCommand = {
	id: string;
	name: string;
	description: string;
	execute: (editor: any) => void;
};

// Define movement commands
export const movementCommands: Record<string, EditorCommand> = {
	moveLeft: {
		id: 'moveLeft',
		name: 'Move Left',
		description: 'Move cursor one character to the left',
		execute: (editor) => {
			if (!editor) return;

			const cursorPos = editor.selectionStart;
			if (cursorPos > 0) {
				editor.setSelectionRange(cursorPos - 1, cursorPos - 1);

				// Ensure the cursor is visible
				const rect = editor.getBoundingClientRect();
				const cursorRect = getCursorRect(editor);

				if (cursorRect && cursorRect.left < rect.left) {
					editor.scrollLeft = Math.max(0, editor.scrollLeft - 20);
				}
			}
		}
	},

	moveRight: {
		id: 'moveRight',
		name: 'Move Right',
		description: 'Move cursor one character to the right',
		execute: (editor) => {
			if (!editor) return;

			const cursorPos = editor.selectionStart;
			const textLength = editor.value.length;

			if (cursorPos < textLength) {
				editor.setSelectionRange(cursorPos + 1, cursorPos + 1);

				// Ensure the cursor is visible
				const rect = editor.getBoundingClientRect();
				const cursorRect = getCursorRect(editor);

				if (cursorRect && cursorRect.right > rect.right) {
					editor.scrollLeft = editor.scrollLeft + 20;
				}
			}
		}
	},

	moveUp: {
		id: 'moveUp',
		name: 'Move Up',
		description: 'Move cursor one line up',
		execute: (editor) => {
			if (!editor) return;

			const text = editor.value;
			const cursorPos = editor.selectionStart;

			// Get all lines
			const allLines = text.split('\n');

			// Find the current line and column
			const textBeforeCursor = text.substring(0, cursorPos);
			const lines = textBeforeCursor.split('\n');
			const currentLine = lines.length - 1;
			const currentColumn = lines[currentLine].length;

			// Can't go up if already at the first line
			if (currentLine <= 0) return;

			// Calculate the position in the previous line
			const previousLineLength = allLines[currentLine - 1].length;
			const targetColumn = Math.min(currentColumn, previousLineLength);

			// Calculate the new cursor position by finding the start of the current line
			// and then going back to the previous line
			let newPosition = 0;
			for (let i = 0; i < currentLine - 1; i++) {
				newPosition += allLines[i].length + 1; // +1 for newline
			}
			newPosition += targetColumn; // Add the target column in the previous line

			// Set the new cursor position
			editor.setSelectionRange(newPosition, newPosition);
		}
	},

	moveDown: {
		id: 'moveDown',
		name: 'Move Down',
		description: 'Move cursor one line down',
		execute: (editor) => {
			if (!editor) return;

			const text = editor.value;
			const cursorPos = editor.selectionStart;

			// Get all lines and find current position
			const allLines = text.split('\n');
			const textBeforeCursor = text.substring(0, cursorPos);
			const linesBefore = textBeforeCursor.split('\n');
			const currentLineIndex = linesBefore.length - 1;

			// Find the current column (visual position)
			const currentColumn = linesBefore[currentLineIndex].length;

			// Can't go down if already at the last line
			if (currentLineIndex >= allLines.length - 1) return;

			// Calculate the position in the next line
			const nextLineLength = allLines[currentLineIndex + 1].length;
			const targetColumn = Math.min(currentColumn, nextLineLength);

			// Calculate the new cursor position
			let newPosition = 0;
			for (let i = 0; i <= currentLineIndex; i++) {
				newPosition += allLines[i].length + 1; // +1 for the newline character
			}
			newPosition += targetColumn;

			// Set the new cursor position
			editor.setSelectionRange(newPosition, newPosition);
		}
	}
};

// Define normal mode keybindings
export const normalModeKeybindings: Record<string, string> = {
	h: 'moveLeft',
	j: 'moveDown',
	k: 'moveUp',
	l: 'moveRight',
	ArrowLeft: 'moveLeft',
	ArrowDown: 'moveDown',
	ArrowUp: 'moveUp',
	ArrowRight: 'moveRight'
};

// Add document switching commands
export const documentCommands: Record<string, EditorCommand> = {
	switchToDocument1: {
		id: 'switchToDocument1',
		name: 'Switch to Document 1',
		description: 'Switch to the first document in the list',
		execute: (editor) => switchToDocumentByIndex(0)
	},
	switchToDocument2: {
		id: 'switchToDocument2',
		name: 'Switch to Document 2',
		description: 'Switch to the second document in the list',
		execute: (editor) => switchToDocumentByIndex(1)
	},
	switchToDocument3: {
		id: 'switchToDocument3',
		name: 'Switch to Document 3',
		description: 'Switch to the third document in the list',
		execute: (editor) => switchToDocumentByIndex(2)
	},
	switchToDocument4: {
		id: 'switchToDocument4',
		name: 'Switch to Document 4',
		description: 'Switch to the fourth document in the list',
		execute: (editor) => switchToDocumentByIndex(3)
	},
	switchToDocument5: {
		id: 'switchToDocument5',
		name: 'Switch to Document 5',
		description: 'Switch to the fifth document in the list',
		execute: (editor) => switchToDocumentByIndex(4)
	},
	switchToDocument6: {
		id: 'switchToDocument6',
		name: 'Switch to Document 6',
		description: 'Switch to the sixth document in the list',
		execute: (editor) => switchToDocumentByIndex(5)
	},
	switchToDocument7: {
		id: 'switchToDocument7',
		name: 'Switch to Document 7',
		description: 'Switch to the seventh document in the list',
		execute: (editor) => switchToDocumentByIndex(6)
	},
	switchToDocument8: {
		id: 'switchToDocument8',
		name: 'Switch to Document 8',
		description: 'Switch to the eighth document in the list',
		execute: (editor) => switchToDocumentByIndex(7)
	},
	switchToDocument9: {
		id: 'switchToDocument9',
		name: 'Switch to Document 9',
		description: 'Switch to the ninth document in the list',
		execute: (editor) => switchToDocumentByIndex(8)
	}
};

// Helper function to get cursor rectangle
function getCursorRect(editor: HTMLTextAreaElement): DOMRect | null {
	// This is a simplified approach - for a real implementation,
	// you might need to create a hidden div with the same styling
	// and measure the text width more precisely
	const selection = window.getSelection();
	if (selection && selection.rangeCount > 0) {
		const range = selection.getRangeAt(0);
		return range.getBoundingClientRect();
	}
	return null;
}

// Helper function to switch to a document by its index
function switchToDocumentByIndex(index: number): void {
	// This function will be called from the global context
	// We need to find the current document switcher component

	// Get all document buttons
	const docButtons = document.querySelectorAll('.doc-button');

	// Check if the requested index is valid
	if (index < docButtons.length) {
		// Get the button for the requested document
		const button = docButtons[index] as HTMLButtonElement;

		// Only click if the button is not disabled (not the current document)
		if (!button.disabled) {
			button.click();
		}
	}
}

// Add Ctrl+Number keybindings to the normal mode keybindings
export const ctrlKeybindings: Record<string, string> = {
	'Control+1': 'switchToDocument1',
	'Control+2': 'switchToDocument2',
	'Control+3': 'switchToDocument3',
	'Control+4': 'switchToDocument4',
	'Control+5': 'switchToDocument5',
	'Control+6': 'switchToDocument6',
	'Control+7': 'switchToDocument7',
	'Control+8': 'switchToDocument8',
	'Control+9': 'switchToDocument9'
};

// Update the handleNormalModeKeydown function to handle Ctrl+Number
export function handleNormalModeKeydown(event: KeyboardEvent, editor: HTMLTextAreaElement): boolean {
	const key = event.key;

	// Handle Ctrl+Number combinations
	if (event.ctrlKey && key >= '1' && key <= '9') {
		const commandId = ctrlKeybindings[`Control+${key}`];
		if (commandId) {
			event.preventDefault();
			return executeCommand(commandId, editor);
		}
	}

	// Check if this key is bound to a movement command
	const commandId = normalModeKeybindings[key];

	if (commandId) {
		event.preventDefault();
		return executeCommand(commandId, editor);
	}

	// Special case for 'i' to enter insert mode
	if (key === 'i') {
		return false; // Let the main handler handle this
	}

	// Prevent default for most keys in normal mode
	if (key.length === 1 && !event.ctrlKey && !event.metaKey) {
		event.preventDefault();
		return true;
	}

	return false;
}

// Update the executeCommand function to handle document commands
export function executeCommand(commandId: string, editor: HTMLTextAreaElement): boolean {
	// Check movement commands
	const movementCommand = movementCommands[commandId];
	if (movementCommand) {
		movementCommand.execute(editor);
		return true;
	}

	// Check document commands
	const documentCommand = documentCommands[commandId];
	if (documentCommand) {
		documentCommand.execute(editor);
		return true;
	}

	return false;
}
