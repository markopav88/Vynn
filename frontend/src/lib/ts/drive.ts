/*
/ Drive.ts
/
/ File containing functions and logic required for frontend handling of a drive
/ Will provide the communication with the backend and pass necessary information to API calls
/
/ Summary:
/
/
/
*/
import { Document } from "./document";
import { Project } from "./project";


// TODO Function to create a document
export async function create_document(document_payload: Document): Promise<Boolean> {
	const apiUrl = `http://localhost:3001/api/document/`;

	try {
		const response = await fetch(apiUrl, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(document_payload),
			credentials: 'include'
		});

		if (response.ok) {
			return true;
		} else {
			console.error('Create document failed with status:', response.status);
			const errorText = await response.text();
			console.error('Error response:', errorText);
			return false;
		}
	} catch (error) {
		console.error('Create document error:', error);
		return false;
	}
}

// TODO Function to get all documents
export async function get_all_documents(): Promise<Document[] | null> {
	const apiUrl = `http://localhost:3001/api/document/`;

	try {
		const response = await fetch(apiUrl, {
			method: 'GET',
			credentials: 'include'
		});

		if (!response.ok) {
			console.error('Get all documents failed with status:', response.status);
			return null;
		}

		const documents = await response.json();
		return documents;
	} catch (error) {
		console.error('Get all documents error:', error);
		return null;
	}
}

// TODO Function to delete document
export async function delete_document(document_id: number): Promise<Boolean> {
	const apiUrl = `http://localhost:3001/api/document/${document_id}`;

	try {
		const response = await fetch(apiUrl, {
			method: 'DELETE',
			credentials: 'include'
		});

		if (response.ok) {
			return true;
		} else {
			console.error('Delete document failed with status:', response.status);
			const errorText = await response.text();
			console.error('Error response:', errorText);
			return false;
		}
	} catch (error) {
		console.error('Delete document error:', error);
		return false;
	}
}

// TODO Function to create a project
export async function create_project(name: string): Promise<Project | null> {
	const apiUrl = `http://localhost:3001/api/project/`;

	try {
		const response = await fetch(apiUrl, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({ _name: name }),
			credentials: 'include'
		});

		if (!response.ok) {
			console.error('Create project failed with status:', response.status);
			return null;
		}

		const project = await response.json();
		return project;
	} catch (error) {
		console.error('Create project error:', error);
		return null;
	}
}

// TODO Function to get a project
export async function get_all_projects(): Promise<Project[] | null> {
	const apiUrl = `http://localhost:3001/api/project/`;

	try {
		const response = await fetch(apiUrl, {
			method: 'GET',
			credentials: 'include'
		});

		if (!response.ok) {
			console.error('Get all projects failed with status:', response.status);
			return null;
		}

		const projects = await response.json();
		return projects;
	} catch (error) {
		console.error('Get all projects error:', error);
		return null;
	}
}

// TODO Function to update a project
export async function update_project(project_id: number, name: string): Promise<Boolean> {
	const apiUrl = `http://localhost:3001/api/project/${project_id}`;
	
	try {
		const response = await fetch(apiUrl, {
			method: 'PUT',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({ _name: name }),
			credentials: 'include'
		});

		if (response.ok) {
			return true;
		} else {
			console.error('Update project failed with status:', response.status);
			return false;
		}
	} catch (error) {
		console.error('Update project error:', error);
		return false;
	}
}

// TODO Function to delete a project
export async function delete_project(project_id: number): Promise<Boolean> {
	const apiUrl = `http://localhost:3001/api/project/${project_id}`;
	
	try {
		const response = await fetch(apiUrl, {
			method: 'DELETE',
			credentials: 'include'
		});

		if (response.ok) {
			return true;
		} else {
			console.error('Delete project failed with status:', response.status);
			return false;
		}
	} catch (error) {
		console.error('Delete project error:', error);
		return false;
	}
}

// TODO Function to delete a project and all its documents
export async function force_delete_project(project_id: number): Promise<Boolean> {
	const apiUrl = `http://localhost:3001/api/project/${project_id}/force`;
	
	try {
		const response = await fetch(apiUrl, {
			method: 'DELETE',
			credentials: 'include'
		});

		if (response.ok) {
			return true;
		} else {
			console.error('Force delete project failed with status:', response.status);
			return false;
		}
	} catch (error) {
		console.error('Force delete project error:', error);
		return false;
	}
}
