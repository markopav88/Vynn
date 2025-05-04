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
	is_starred: boolean;
	is_trashed: boolean;
	project_id?: number;

	constructor(
		new_id: number,
		new_name: string,
		new_content: string,
		new_created_at: string,
		new_updated_at: string,
		new_is_starred: boolean = false,
		new_is_trashed: boolean = false,
		new_project_id?: number
	) {
		this.id = new_id;
		this.name = new_name;
		this.content = new_content;
		this.created_at = new_created_at;
		this.updated_at = new_updated_at;
		this.is_starred = new_is_starred;
		this.is_trashed = new_is_trashed;
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
 * Function to get a document by ID
 * Calls: GET /api/document/:id
 * Test: test_documents.rs/test_get_document()
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
 * Test: test_documents.rs/test_update_document()
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
 * Test: test_documents.rs/test_get_permissions()
 */
export async function get_document_permissions(document_id: number): Promise<DocumentUser[] | null> {
	try {
		const apiUrl = `http://localhost:3001/api/document/${document_id}/permissions`;

		const response = await fetch(apiUrl, {
			credentials: 'include'
		});

		if (!response.ok) {
			throw new Error(`Failed to fetch document users: ${response.statusText}`);
		}

		const data = await response.json();
		return data || null;
	} catch (error) {
		console.error('Error loading document users:', error);
		return null;
	}
}

/**
 * Function to get all documents the user has access to
 * Calls: GET /api/document
 * Test: test_documents.rs/test_get_all_documents()
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
 * Test: test_documents.rs/test_create_document()
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
 * Test: test_documents.rs/test_delete_document()
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
 * Test: test_documents.rs/test_add_permissions()
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
 * Test: test_documents.rs/test_update_permission()
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

		if (!response.ok) {
			const errorText = await response.text();
			throw new Error(errorText || 'Failed to update document permissions');
		}

		return true;
	} catch (error) {
		console.error('Error updating document permissions:', error);
		throw error; // Re-throw the error to be handled by the caller
	}
}

/**
 * Function to delete a user's permissions for a document
 * Calls: DELETE /api/document/:id/permissions/:user_id
 * Test: test_documents.rs/test_remove_permissions()
 */
export async function delete_document_permissions(documentId: number, userId: number): Promise<boolean> {
	try {
		const apiUrl = `http://localhost:3001/api/document/${documentId}/permissions/${userId}`;

		const response = await fetch(apiUrl, {
			method: 'DELETE',
			headers: {
				'Content-Type': 'application/json'
			},
			credentials: 'include'
		});

		if (!response.ok) {
			const errorText = await response.text();
			throw new Error(errorText || 'Failed to remove user from document');
		}

		return true;
	} catch (error) {
		console.error('Error deleting document permissions:', error);
		throw error; // Re-throw the error to be handled by the caller
	}
}

/**
 * Function to get the project associated with a document
 * Calls: GET /api/document/:id/project
 * Test: test_documents.rs/test_get_project_from_document()
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

/**
 * Function to toggle 'starred' status of a document
 * Calls: PUT /api/document/:id/star
 * Test: TODO: test_documents.rs/test_toggle_star_document() - Test missing
 */
export async function toggle_star_document(document: Document): Promise<boolean> {
	try {
		const apiUrl = `http://localhost:3001/api/document/${document.id}/star`;

		const response = await fetch(apiUrl, {
			method: 'PUT',
			credentials: 'include'
		});

		return response.ok;
	} catch (error) {
		console.error('Error toggling star status:', error);
		return false;
	}
}

/**
 * Function to move a document to trash
 * Calls: PUT /api/document/:id/trash
 * Test: TODO: test_documents.rs/test_trash_document() - Test missing
 */
export async function trash_document(document: Document): Promise<boolean> {
	try {
		const apiUrl = `http://localhost:3001/api/document/${document.id}/trash`;

		const response = await fetch(apiUrl, {
			method: 'PUT',
			credentials: 'include'
		});

		return response.ok;
	} catch (error) {
		console.error('Error trashing document:', error);
		return false;
	}
}

/**
 * Function to restore a document from trash
 * Calls: PUT /api/document/:id/restore
 * Test: TODO: test_documents.rs/test_restore_document() - Test missing
 */
export async function restore_document(document: Document): Promise<boolean> {
	try {
		const apiUrl = `http://localhost:3001/api/document/${document.id}/restore`;

		const response = await fetch(apiUrl, {
			method: 'PUT',
			credentials: 'include'
		});

		return response.ok;
	} catch (error) {
		console.error('Error restoring document:', error);
		return false;
	}
}

/**
 * Function to get all starred documents
 * Calls: GET /api/document/starred
 * Test: TODO: test_documents.rs/test_get_starred_documents() - Test missing
 */
export async function get_starred_documents(): Promise<Document[] | null> {
	try {
		const apiUrl = `http://localhost:3001/api/document/starred`;

		const response = await fetch(apiUrl, {
			credentials: 'include'
		});

		if (!response.ok) {
			console.error('Failed to fetch starred documents:', response.status);
			return null;
		}

		return await response.json();
	} catch (error) {
		console.error('Error fetching starred documents:', error);
		return null;
	}
}

/**
 * Function to get all trashed documents
 * Calls: GET /api/document/trash
 * Test: TODO: test_documents.rs/test_get_trashed_documents() - Test missing
 */
export async function get_trashed_documents(): Promise<Document[] | null> {
	try {
		const apiUrl = `http://localhost:3001/api/document/trash`;

		const response = await fetch(apiUrl, {
			credentials: 'include'
		});

		if (!response.ok) {
			console.error('Failed to fetch trashed documents:', response.status);
			return null;
		}

		return await response.json();
	} catch (error) {
		console.error('Error fetching trashed documents:', error);
		return null;
	}
}

/**
 * Function to get all shared documents
 * Calls: GET /api/document/shared
 * Test: TODO: test_documents.rs/test_get_shared_documents() - Test missing
 */
export async function get_shared_documents(): Promise<Document[] | null> {
	const apiUrl = `http://localhost:3001/api/document/shared`;

	try {
		const response = await fetch(apiUrl, {
			method: 'GET',
			credentials: 'include',
			headers: {
				'Content-Type': 'application/json'
			}
		});

		if (!response.ok) {
			console.error('Get shared documents failed with status:', response.status);
			const errorText = await response.text();
			console.error('Error response:', errorText);
			return null;
		}

		const documents = await response.json();
		return documents;
	} catch (error) {
		console.error('Get shared documents error:', error);
		return null;
	}
}
