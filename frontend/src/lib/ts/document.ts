/*
/ Document.ts
/
/ File containing functions and logic required for frontend handling of documents
/ Will provide the communication with the backend and pass necessary information to API calls
/
/ Content:
/ Class Document: Mapper of a class to how we are storing documents in db
/ loadDocument: Function ran on mount of /document/:id that will call GET API
/ updateDocument: Function ran every 30 seconds while within a document that updates db instance
/
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
export async function loadDocument(documentId: number): Promise<Document | null> {
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
		} catch (e) {
			console.error('Error parsing document data:', e);
			return null;
		}
	} catch (e) {
		console.error('Error loading document:', e);
		return null;
	}
}

// Function to take the current state of the document and update it in the database
export function updateDocument(document: Document) {
		// Call update API with current document
}