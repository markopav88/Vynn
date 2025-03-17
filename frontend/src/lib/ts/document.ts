/*
/ Document.ts
/
/ File containing functions and logic required for frontend handling of documents
/ Will provide the communication with the backend and pass necessary information to API calls
/
/ Summary:
/ Class Document: Mapper of a class to how we are storing documents in db
/ load_document: Function ran on mount of /document/:id that will call GET API
/ update_document: Function to call update document POST API and pass in new document state
/ setup_auto_save: Function to setup interval of 30 seconds for auto-save 
/
/
*/

export class Document {
	id: number;
	name: string;
	content: string;
	created_at: string;
	updated_at: string;

	constructor(
		new_id: number,
		new_name: string,
		new_content: string,
		new_created_at: string,
		new_updated_at: string
	) {
		this.id = new_id;
		this.name = new_name;
		this.content = new_content;
		this.created_at = new_created_at;
		this.updated_at = new_updated_at;
	}
}

// Function to parse the saved document state into how it is supposed to look
export async function load_document(documentId: number): Promise<Document | null> {
	try {
		// Use the correct backend API URL
		const apiUrl = `http://localhost:3001/api/document/${documentId}`;

		const response = await fetch(apiUrl);

		if (!response.ok) {
			throw new Error(`Failed to fetch document: ${response.statusText}`);
		}

		// Check if the response is JSON
		const contentType = response.headers.get('Content-Type');
		if (!contentType || !contentType.includes('application/json')) {
			// If the response is not JSON, log it and return null
			const text = await response.text(); // Read the response as text to inspect it
			console.error('Expected JSON, but received:', text);
			return null;
		}

		// Parse the response JSON
		const data = await response.json();

		// Parse Document
		try {
			let document = new Document(
				data.id,
				data.name,
				data.content || "", // Handle null content
				data.created_at,
				data.updated_at
			);
			return document;
		} catch (error) {
			console.error('Error parsing document data:', error);
			return null;
		}
	} catch (error) {
		console.error('Error loading document:', error);
		return null;
	}
}

// Function to take the current state of the document and update it in the database
export async function update_document(document: Document): Promise<boolean> {
	try {
		// Use the correct backend API URL
		const apiUrl = `http://localhost:3001/api/document/${document.id}`;
		
		// Format the timestamp in the format expected by the backend (NaiveDateTime)
		const now = new Date().toISOString().replace('Z', '');
		
		// Create payload with explicit content handling
		const payload = {
			name: document.name,
			content: document.content || "", // Ensure content is never null/undefined
			updated_at: now
		};
		
		console.log("Sending update with payload:", payload);
		
		const response = await fetch(apiUrl, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
			},
			body: JSON.stringify(payload),
		});
		
		if (!response.ok) {
			const errorText = await response.text();
			console.error("Update failed:", response.status, errorText);
			return false;
		}
		
		console.log("Document updated successfully");
		return true;
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
export async function saveDocument(documentData: Document): Promise<boolean | null> {
	if (documentData) {
		return await update_document(documentData);
	}
	return null;
}