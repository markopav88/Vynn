/*
/ Drive.ts
/
/ File containing functions and logic required for frontend handling of a drive
/ Will provide the communication with the backend and pass necessary information to API calls
/
/ Summary:
/ Interface Project: Structure for project data
/ create_document: Function to create a new document
/ get_all_documents: Function to get all documents the user has access to
/ delete_document: Function to delete a document by ID
/ create_project: Function to create a new project
/ get_all_projects: Function to get all projects for the current user
/ update_project: Function to update a project by ID
/ delete_project: Function to delete a project by ID
/ force_delete_project: Function to force delete a project and all its documents
/
*/
import { Document } from './document';

// Define and export the Project interface
export interface Project {
	id: string;
	name: string;
	description?: string;
	created_at: string;
	updated_at: string;
	is_starred: boolean;
	is_trashed: boolean;
	user_id: string;
}

/**
 * Function to create a document
 * Calls: POST /api/document
 */
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

/**
 * Function to get all documents the user has access to
 * Calls: GET /api/document
 */
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

/**
 * Function to delete a document
 * Calls: DELETE /api/document/:id
 */
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

/**
 * Function to create a project
 * Calls: POST /api/project
 */
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

/**
 * Function to get all projects for the current user
 * Calls: GET /api/project
 */
export async function get_all_projects(): Promise<Project[] | null> {
	const apiUrl = `http://localhost:3001/api/project`;

	try {
		console.log('Fetching projects from:', apiUrl);

		const response = await fetch(apiUrl, {
			method: 'GET',
			credentials: 'include',
			headers: {
				Accept: 'application/json'
			}
		});

		console.log('Project response status:', response.status);

		if (!response.ok) {
			console.error('Get all projects failed with status:', response.status);
			const errorText = await response.text();
			console.error('Error response:', errorText);
			return null;
		}

		const projects = await response.json();
		console.log('Projects received:', projects);
		return projects;
	} catch (error) {
		console.error('Get all projects error:', error);
		return null;
	}
}

/**
 * Function to update a project
 * Calls: PUT /api/project/:id
 */
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

/**
 * Function to delete a project
 * Calls: DELETE /api/project/:id
 */
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

/**
 * Function to force delete a project and all its documents
 * Calls: DELETE /api/project/:id/force
 */
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

/**
 * Function to toggle 'starred' status of a project
 */
export async function toggle_star_project(project_id: number): Promise<Boolean> {
	const apiUrl = `http://localhost:3001/api/project/${project_id}/star`;

	try {
		const response = await fetch(apiUrl, {
			method: 'PUT',
			credentials: 'include'
		});

		if (response.ok) {
			return true;
		} else {
			console.error('Toggle star project failed with status:', response.status);
			return false;
		}
	} catch (error) {
		console.error('Toggle star project error:', error);
		return false;
	}
}

/**
 * Function to move a project to trash
 */
export async function trash_project(project_id: number): Promise<Boolean> {
	const apiUrl = `http://localhost:3001/api/project/${project_id}/trash`;

	try {
		const response = await fetch(apiUrl, {
			method: 'PUT',
			credentials: 'include'
		});

		if (response.ok) {
			return true;
		} else {
			console.error('Trash project failed with status:', response.status);
			return false;
		}
	} catch (error) {
		console.error('Trash project error:', error);
		return false;
	}
}

/**
 * Function to restore a project from trash
 */
export async function restore_project(project_id: number): Promise<Boolean> {
	const apiUrl = `http://localhost:3001/api/project/${project_id}/restore`;

	try {
		const response = await fetch(apiUrl, {
			method: 'PUT',
			credentials: 'include'
		});

		if (response.ok) {
			return true;
		} else {
			console.error('Restore project failed with status:', response.status);
			return false;
		}
	} catch (error) {
		console.error('Restore project error:', error);
		return false;
	}
}

/**
 * Function to get all starred projects
 */
export async function get_starred_projects(): Promise<Project[] | null> {
	const apiUrl = `http://localhost:3001/api/project/starred`;

	try {
		const response = await fetch(apiUrl, {
			credentials: 'include'
		});

		if (!response.ok) {
			console.error('Get starred projects failed with status:', response.status);
			return null;
		}

		const projects = await response.json();
		return projects;
	} catch (error) {
		console.error('Get starred projects error:', error);
		return null;
	}
}

/**
 * Function to get all trashed projects
 */
export async function get_trashed_projects(): Promise<Project[] | null> {
	const apiUrl = `http://localhost:3001/api/project/trash`;

	try {
		const response = await fetch(apiUrl, {
			credentials: 'include'
		});

		if (!response.ok) {
			console.error('Get trashed projects failed with status:', response.status);
			return null;
		}

		const projects = await response.json();
		return projects;
	} catch (error) {
		console.error('Get trashed projects error:', error);
		return null;
	}
}
