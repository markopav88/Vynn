/*
/ Project.ts
/
/ File containing functions and logic required for frontend handling of projects
/ Will provide the communication with the backend and pass necessary information to API calls
/
/ Summary:
/ Class Project: Mapper of a class to how we are storing projects in db
/ Class ProjectUser: Represents a user with project permissions
/ get_project: Function to get a project by ID
/ update_project: Function to update a project
/ delete_project: Function to delete a project
/ force_delete_project: Function to delete a project and all documents in it
/ add_document_to_project: Function to add a document to a project
/ remove_document_from_project: Function to remove a document from a project
/ get_project_documents: Function to get all documents in a project
/ add_project_permissions: Function to add permissions for a user on a project
/ update_project_permissions: Function to update a user's permissions for a project
/ delete_project_permissions: Function to delete a user's permissions for a project
/ get_project_permissions: Function to get all users with permissions for a project
/
*/

import type { Document } from './document';

const API_BASE_URL = process.env.API_BASE_URL;

export class Project {
	id: number;
	name: string;
	user_id?: number;

	constructor(new_id: number, new_name: string, new_user_id?: number) {
		this.id = new_id;
		this.name = new_name;
		this.user_id = new_user_id;
	}
}

// Define a User type for project permissions
export class ProjectUser {
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
 * Function to get a project by ID
 * Calls: GET /api/project/:id
 */
export async function get_project(project_id: number): Promise<Project | null> {
	const apiUrl = `${API_BASE_URL}/api/project/${project_id}`;

	try {
		const response = await fetch(apiUrl, {
			method: 'GET',
			credentials: 'include'
		});

		if (!response.ok) {
			console.error('Get project failed with status:', response.status);
			return null;
		}

		const project = await response.json();
		return project;
	} catch (error) {
		console.error('Get project error:', error);
		return null;
	}
}

/**
 * Function to get all projects for the current user
 * Calls: GET /api/project
 */
export async function get_all_projects(): Promise<Project[] | null> {
	const apiUrl = `${API_BASE_URL}/api/project`;

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

/**
 * Function to update a project
 * Calls: PUT /api/project/:id
 */
export async function update_project(project_id: number, name: string): Promise<boolean> {
	const apiUrl = `${API_BASE_URL}/api/project/${project_id}`;

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
export async function delete_project(project_id: number): Promise<boolean> {
	const apiUrl = `${API_BASE_URL}/api/project/${project_id}`;

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
export async function force_delete_project(project_id: number): Promise<boolean> {
	const apiUrl = `${API_BASE_URL}/api/project/${project_id}/force`;

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
 * Function to add permissions for a user on a project
 * Calls: POST /api/project/:id/permissions
 */
export async function add_project_permissions(projectId: number, userId: number, role: string): Promise<boolean> {
	try {
		const apiUrl = `${API_BASE_URL}/api/project/${projectId}/permissions`;

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
		console.error('Error adding project permissions:', error);
		return false;
	}
}

/**
 * Function to get all users with permissions on a project
 * Calls: GET /api/project/:id/permissions
 */
export async function get_project_permissions(projectId: number): Promise<ProjectUser[] | null> {
	try {
		const apiUrl = `${API_BASE_URL}/api/project/${projectId}/permissions`;

		const response = await fetch(apiUrl, {
			credentials: 'include'
		});

		if (!response.ok) {
			console.error('Failed to fetch project permissions:', response.status);
			return null;
		}

		const data = await response.json();
		return data || null;
	} catch (error) {
		console.error('Error fetching project permissions:', error);
		return null;
	}
}

/**
 * Function to update a user's permission on a project
 * Calls: PUT /api/project/:id/permissions
 */
export async function update_project_permission(projectId: number, userId: number, role: string): Promise<boolean> {
	try {
		const apiUrl = `${API_BASE_URL}/api/project/${projectId}/permissions`;

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
			throw new Error(errorText || 'Failed to update project permissions');
		}

		return true;
	} catch (error) {
		console.error('Error updating project permission:', error);
		throw error;
	}
}

/**
 * Function to remove a user's permission from a project
 * Calls: DELETE /api/project/:id/permissions/:user_id
 */
export async function remove_project_permissions(projectId: number, userId: number): Promise<boolean> {
	try {
		const apiUrl = `${API_BASE_URL}/api/project/${projectId}/permissions/${userId}`;

		const response = await fetch(apiUrl, {
			method: 'DELETE',
			headers: {
				'Content-Type': 'application/json'
			},
			credentials: 'include'
		});

		if (!response.ok) {
			const errorText = await response.text();
			throw new Error(errorText || 'Failed to remove user from project');
		}

		return true;
	} catch (error) {
		console.error('Error removing project permissions:', error);
		throw error;
	}
}

/**
 * Function to get all documents in a project
 * Calls: GET /api/project/:id/documents
 */
export async function get_project_documents(project_id: number): Promise<Document[] | null> {
	const apiUrl = `${API_BASE_URL}/api/project/${project_id}/documents`;

	try {
		const response = await fetch(apiUrl, {
			credentials: 'include'
		});

		if (!response.ok) {
			console.error('Failed to fetch project documents:', response.status);
			return null;
		}

		const data = await response.json();
		console.log('Project documents received:', data);

		return data || [];
	} catch (error) {
		console.error('Error fetching project documents:', error);
		return null;
	}
}

/**
 * Function to add a document to a project
 * Calls: POST /api/project/:id/documents/:doc_id
 */
export async function add_document_to_project(projectId: number, documentId: number): Promise<boolean> {
	const apiUrl = `${API_BASE_URL}/api/project/${projectId}/documents/${documentId}`;

	try {
		const response = await fetch(apiUrl, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({}),
			credentials: 'include'
		});

		if (response.ok) {
			// Log success for debugging
			console.log(`Document ${documentId} successfully added to project ${projectId}`);
			return true;
		} else {
			console.error(`Failed to add document ${documentId} to project ${projectId}:`, response.status);
			return false;
		}
	} catch (error) {
		console.error('Error adding document to project:', error);
		return false;
	}
}

/**
 * Function to remove a document from a project
 * Calls: DELETE /api/project/:id/documents/:doc_id
 */
export async function remove_document_from_project(projectId: number, documentId: number): Promise<boolean> {
	const apiUrl = `${API_BASE_URL}/api/project/${projectId}/documents/${documentId}`;

	try {
		const response = await fetch(apiUrl, {
			method: 'DELETE',
			credentials: 'include'
		});

		return response.ok;
	} catch (error) {
		console.error('Error removing document from project:', error);
		return false;
	}
}
