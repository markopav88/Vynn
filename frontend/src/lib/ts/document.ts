/*
/ Document.ts
/
/ File containing functions and logic required for frontend handling of documents and keybindings
/ Will provide the communication with the backend and pass necessary information to API calls
/
/ Summary:
/ Class Document: Mapper of a class to how we are storing documents in db
/ Class DocumentUser: Represents a user with document permissions
/ Class Command: Represents a command from the database
/ Class UserKeybinding: Represents a user's custom keybinding
/ 
/ Document Functions:
/ get_document: Function to get a document by ID
/ update_document: Function to update a document
/ setup_auto_save: Function to setup interval of 30 seconds for auto-save
/ saveDocument: Manual save function for when we want to bind this
/ delete_document: Function to delete a document
/ add_document_permissions: Function to add permissions for a user on a document
/ update_document_permissions: Function to update a user's permissions for a document
/ delete_document_permissions: Function to delete a user's permissions for a document
/
/ Keybinding Functions:
/ get_all_commands: Function to get all available commands
/ get_all_keybindings: Function to get all user's custom keybindings
/ add_update_keybinding: Function to add or update a keybinding
/ delete_keybinding: Function to delete a keybinding (reset to default)
/
*/

export class Document {
	id: number;
	name: string;
	content: string;
	created_at: string;
	updated_at: string;
	project_id?: number;

	constructor(
		new_id: number,
		new_name: string,
		new_content: string,
		new_created_at: string,
		new_updated_at: string,
		new_project_id?: number
	) {
		this.id = new_id;
		this.name = new_name;
		this.content = new_content;
		this.created_at = new_created_at;
		this.updated_at = new_updated_at;
		this.project_id = new_project_id;
	}
}

// Define a User type for document permissions
export class DocumentUser {
	id: number;
	name: string;
	email: string;
	role: string;

	constructor(new_id: number, new_name: string, new_email: string, new_role: string) {
		this.id = new_id;
		this.name = new_name;
		this.email = new_email;
		this.role = new_role;
	}
}

/**
 * Command class representing a command from the database
 */
export class Command {
	command_id: number;
	command_name: string;
	command_description: string;
	default_keybinding: string;

	constructor(id: number, name: string, description: string, default_keybinding: string) {
		this.command_id = id;
		this.command_name = name;
		this.command_description = description;
		this.default_keybinding = default_keybinding;
	}
}

/**
 * UserKeybinding class representing a user's custom keybinding
 */
export class UserKeybinding {
	user_id: number;
	command_id: number;
	keybinding: string;

	constructor(user_id: number, command_id: number, keybinding: string) {
		this.user_id = user_id;
		this.command_id = command_id;
		this.keybinding = keybinding;
	}
}

/**
 * Function to get a document by ID
 * Calls: GET /api/document/:id
 */
export async function get_document(id: number): Promise<Document | null> {
	try {
		// Use the original endpoint that was working before
		const apiUrl = `http://localhost:3001/api/document/${id}`;

		const response = await fetch(apiUrl, {
			credentials: 'include'
		});

		if (!response.ok) {
			console.error('Failed to fetch document:', response.status);
			return null;
		}

		const data = await response.json();
		console.log('Document data received:', data);
		return data;
	} catch (error) {
		console.error('Error fetching document:', error);
		return null;
	}
}

/**
 * Function to update a document
 * Calls: PUT /api/document/:id
 */
export async function update_document(document: Document): Promise<boolean> {
	try {
		const apiUrl = `http://localhost:3001/api/document/${document.id}`;

		const payload = {
			name: document.name,
			content: document.content,
			updated_at: new Date().toISOString().replace('Z', '')
		};

		const response = await fetch(apiUrl, {
			method: 'PUT',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(payload),
			credentials: 'include'
		});

		return response.ok;
	} catch (error) {
		console.error('Error updating document:', error);
		return false;
	}
}

// Function to set up auto-save interval for a document
export function setup_auto_save(document: Document, onSave?: (success: boolean) => void): () => void {
	// Set up interval to save every 30 seconds
	const intervalId = setInterval(async () => {
		console.log('Auto-saving document...');
		const success = await update_document(document);

		if (onSave) {
			onSave(success);
		}

		if (success) {
			console.log('Document saved successfully');
		} else {
			console.error('Failed to save document');
		}
	}, 30000); // 30 seconds in milliseconds

	// Return a cleanup function to clear the interval
	return () => {
		clearInterval(intervalId);
		console.log('Auto-save disabled');
	};
}

// Manual save function for when we want to bind this
export async function saveDocument(document: Document): Promise<boolean | null> {
	if (document) {
		return await update_document(document);
	}
	return null;
}

/**
 * Function to get all users with permissions to a given document
 * Calls: GET /api/document/:id/permissions
 */
export async function get_document_permissions(documentData: Document): Promise<DocumentUser[] | null> {
	try {
		const apiUrl = `http://localhost:3001/api/document/${documentData.id}/permissions`;

		const response = await fetch(apiUrl, {
			credentials: 'include'
		});

		if (!response.ok) {
			throw new Error(`Failed to fetch document users: ${response.statusText}`);
		}

		const data = await response.json();
		return data.users || null;
	} catch (error) {
		console.error('Error loading document users:', error);
		return null;
	}
}

/**
 * Function to get all documents the user has access to
 * Calls: GET /api/document
 */
export async function get_all_documents(): Promise<Document[] | null> {
	try {
		const apiUrl = `http://localhost:3001/api/document`;

		console.log('Fetching documents from:', apiUrl);

		const response = await fetch(apiUrl, {
			method: 'GET',
			credentials: 'include',
			headers: {
				Accept: 'application/json'
			}
		});

		console.log('Document response status:', response.status);

		if (!response.ok) {
			console.error('Failed to fetch documents:', response.status);
			const errorText = await response.text();
			console.error('Error response:', errorText);
			return null;
		}

		const documents = await response.json();
		console.log('Documents received:', documents);
		return documents;
	} catch (error) {
		console.error('Error fetching documents:', error);
		return null;
	}
}

/**
 * Function to create a new document
 * Calls: POST /api/document
 */
export async function create_document(name: string, content: string): Promise<Document | null> {
	try {
		const apiUrl = `http://localhost:3001/api/document`;
		const now = new Date().toISOString().replace('Z', '');

		const payload = {
			name: name,
			content: content,
			created_at: now,
			updated_at: now
		};

		const response = await fetch(apiUrl, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(payload),
			credentials: 'include'
		});

		if (!response.ok) {
			console.error('Failed to create document:', response.status);
			return null;
		}

		return await response.json();
	} catch (error) {
		console.error('Error creating document:', error);
		return null;
	}
}

/**
 * Function to delete a document
 * Calls: DELETE /api/document/:id
 */
export async function delete_document(documentId: number): Promise<boolean> {
	try {
		const apiUrl = `http://localhost:3001/api/document/${documentId}`;

		const response = await fetch(apiUrl, {
			method: 'DELETE',
			credentials: 'include'
		});

		return response.ok;
	} catch (error) {
		console.error('Error deleting document:', error);
		return false;
	}
}

/**
 * Function to add permissions for a user on a document
 * Calls: POST /api/document/:id/permissions
 */
export async function add_document_permissions(documentId: number, userId: number, role: string): Promise<boolean> {
	try {
		const apiUrl = `http://localhost:3001/api/document/${documentId}/permissions`;

		const payload = {
			user_id: userId,
			role: role
		};

		const response = await fetch(apiUrl, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(payload),
			credentials: 'include'
		});

		return response.ok;
	} catch (error) {
		console.error('Error adding document permissions:', error);
		return false;
	}
}

/**
 * Function to update a user's permissions for a document
 * Calls: PUT /api/document/:id/permissions
 */
export async function update_document_permissions(documentId: number, userId: number, role: string): Promise<boolean> {
	try {
		const apiUrl = `http://localhost:3001/api/document/${documentId}/permissions`;

		const payload = {
			user_id: userId,
			role: role
		};

		const response = await fetch(apiUrl, {
			method: 'PUT',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(payload),
			credentials: 'include'
		});

		return response.ok;
	} catch (error) {
		console.error('Error updating document permissions:', error);
		return false;
	}
}

/**
 * Function to delete a user's permissions for a document
 * Calls: DELETE /api/document/:id/permissions/:user_id
 */
export async function delete_document_permissions(documentId: number, userId: number): Promise<boolean> {
	try {
		const apiUrl = `http://localhost:3001/api/document/${documentId}/permissions/${userId}`;

		const response = await fetch(apiUrl, {
			method: 'DELETE',
			credentials: 'include'
		});

		return response.ok;
	} catch (error) {
		console.error('Error deleting document permissions:', error);
		return false;
	}
}

/**
 * Function to get all available commands
 * Calls: GET /api/command/default
 */
export async function get_all_commands(): Promise<Command[] | null> {
	try {
		const apiUrl = `http://localhost:3001/api/command/default`;

		const response = await fetch(apiUrl, {
			credentials: 'include'
		});

		if (!response.ok) {
			console.error('Failed to fetch commands:', response.status);
			return null;
		}

		return await response.json();
	} catch (error) {
		console.error('Error fetching commands:', error);
		return null;
	}
}

/**
 * Function to get all user's custom keybindings
 * Calls: GET /api/command
 */
export async function get_all_keybindings(): Promise<UserKeybinding[] | null> {
	try {
		const apiUrl = `http://localhost:3001/api/command`;

		const response = await fetch(apiUrl, {
			credentials: 'include'
		});

		if (!response.ok) {
			console.error('Failed to fetch keybindings:', response.status);
			return null;
		}

		return await response.json();
	} catch (error) {
		console.error('Error fetching keybindings:', error);
		return null;
	}
}

/**
 * Function to add or update a keybinding
 * Calls: PUT /api/command/:id
 */
export async function add_update_keybinding(commandId: number, keybinding: string): Promise<UserKeybinding | null> {
	try {
		const apiUrl = `http://localhost:3001/api/command/${commandId}`;

		const response = await fetch(apiUrl, {
			method: 'PUT',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({ keybinding }),
			credentials: 'include'
		});

		if (!response.ok) {
			console.error('Failed to update keybinding:', response.status);
			return null;
		}

		return await response.json();
	} catch (error) {
		console.error('Error updating keybinding:', error);
		return null;
	}
}

/**
 * Function to delete a keybinding (reset to default)
 * Calls: DELETE /api/command/:id
 */
export async function delete_keybinding(commandId: number): Promise<Command | null> {
	try {
		const apiUrl = `http://localhost:3001/api/command/${commandId}`;

		const response = await fetch(apiUrl, {
			method: 'DELETE',
			credentials: 'include'
		});

		if (!response.ok) {
			console.error('Failed to delete keybinding:', response.status);
			return null;
		}

		return await response.json();
	} catch (error) {
		console.error('Error deleting keybinding:', error);
		return null;
	}
}

/**
 * Function to get the project associated with a document
 * Calls: GET /api/document/:id/project
 */
export async function get_project_from_document(
	documentId: number
): Promise<{ project_id: number; project_name: string } | null> {
	try {
		const apiUrl = `http://localhost:3001/api/document/${documentId}/project`;

		const response = await fetch(apiUrl, {
			credentials: 'include'
		});

		if (!response.ok) {
			console.error('Failed to fetch project from document:', response.status);
			return null;
		}

		const data = await response.json();
		console.log('Project data received:', data);
		return data;
	} catch (error) {
		console.error('Error fetching project from document:', error);
		return null;
	}
}
