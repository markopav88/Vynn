/*
/ Project.ts
/
/ File containing functions and logic required for frontend handling of projects
/ Will provide the communication with the backend and pass necessary information to API calls
/
/ Summary:
/ Class Project: Mapper of a class to how we are storing projects in db
/ get_all_projects: Function to get all projects for the current user
/ get_project: Function to get a specific project by ID
/ create_project: Function to create a new project
/ update_project: Function to update an existing project
/ delete_project: Function to delete a project
/ force_delete_project: Function to force delete a project and all its documents
/ add_project_permissions: Function to add permissions for a user on a project
/ get_project_permissions: Function to get all users with permissions on a project
/ update_project_permission: Function to update a user's permission on a project
/ remove_project_permissions: Function to remove a user's permission from a project
/ get_project_documents: Function to get all documents in a project
/ add_document_to_project: Function to add a document to a project
/ remove_document_from_project: Function to remove a document from a project
/
*/

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
 * Function to get all projects for the current user
 * TODO: Implement function to fetch all projects from the backend
 */
export async function get_all_projects(): Promise<Project[] | null> {
	// TODO: Implement API call to GET /api/project
	return null;
}

/**
 * Function to get a specific project by ID
 * TODO: Implement function to fetch a project by ID from the backend
 */
export async function get_project(projectId: number): Promise<Project | null> {
	// TODO: Implement API call to GET /api/project/:id
	return null;
}

/**
 * Function to create a new project
 * TODO: Implement function to create a new project in the backend
 */
export async function create_project(name: string): Promise<Project | null> {
	// TODO: Implement API call to POST /api/project
	return null;
}

/**
 * Function to update an existing project
 * TODO: Implement function to update a project in the backend
 */
export async function update_project(project: Project): Promise<Project | null> {
	// TODO: Implement API call to PUT /api/project/:id
	return null;
}

/**
 * Function to delete a project
 * TODO: Implement function to delete a project from the backend
 */
export async function delete_project(projectId: number): Promise<boolean> {
	// TODO: Implement API call to DELETE /api/project/:id
	return false;
}

/**
 * Function to force delete a project and all its documents
 * TODO: Implement function to force delete a project and all its documents
 */
export async function force_delete_project(projectId: number): Promise<boolean> {
	// TODO: Implement API call to DELETE /api/project/:id/force
	return false;
}

/**
 * Function to add permissions for a user on a project
 * TODO: Implement function to add permissions for a user on a project
 */
export async function add_project_permissions(project_user: ProjectUser): Promise<boolean> {
	// TODO: Implement API call to POST /api/project/:id/permissions
	return false;
}

/**
 * Function to get all users with permissions on a project
 * TODO: Implement function to get all users with permissions on a project
 */
export async function get_project_permissions(projectId: number): Promise<ProjectUser[] | null> {
	// TODO: Implement API call to GET /api/project/:id/permissions
	return null;
}

/**
 * Function to update a user's permission on a project
 * TODO: Implement function to update a user's permission on a project
 */
export async function update_project_permission(
	projectId: number,
	userId: number,
	role: string
): Promise<boolean> {
	// TODO: Implement API call to PUT /api/project/:id/permissions
	return false;
}

/**
 * Function to remove a user's permission from a project
 * TODO: Implement function to remove a user's permission from a project
 */
export async function remove_project_permissions(
	projectId: number,
	userId: number
): Promise<boolean> {
	// TODO: Implement API call to DELETE /api/project/:id/permissions/:user_id
	return false;
}

/**
 * Function to get all documents in a project
 * TODO: Implement function to get all documents in a project
 */
export async function get_project_documents(projectId: number): Promise<any[] | null> {
	// TODO: Implement API call to GET /api/project/:id/documents
	return null;
}

/**
 * Function to add a document to a project
 * TODO: Implement function to add a document to a project
 */
export async function add_document_to_project(
	projectId: number,
	documentId: number
): Promise<boolean> {
	// TODO: Implement API call to POST /api/project/:id/documents/:doc_id
	return false;
}

/**
 * Function to remove a document from a project
 * TODO: Implement function to remove a document from a project
 */
export async function remove_document_from_project(
	projectId: number,
	documentId: number
): Promise<boolean> {
	// TODO: Implement API call to DELETE /api/project/:id/documents/:doc_id
	return false;
}
