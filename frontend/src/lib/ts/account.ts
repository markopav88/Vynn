// frontend/src/lib/ts/account.ts
/*
/ account.ts
/
/ File containing functions and logic required for frontend handling of user accounts and keybindings
/ Will provide the communication with the backend and pass necessary information to API calls
/
/ Summary:
/ Class Command: Represents a command from the database (used for keybindings)
/ Class UserKeybinding: Represents a user's custom keybinding
/ 
/ Keybinding Functions:
/ get_all_keybindings: Function to get all user's custom keybindings
/ add_update_keybinding: Function to add or update a keybinding
/ delete_keybinding: Function to delete a keybinding (reset to default)
/
*/

const API_BASE_URL = process.env.API_BASE_URL;

// --- Classes related to Keybindings --- 
/**
 * Command class representing a command from the database (used for keybindings)
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

// --- API Functions for Keybindings --- 

/**
 * Function to get all user's custom keybindings
 * Calls: GET /api/command  <- TODO: Update endpoint path if necessary (e.g., /api/keybinding)
 */
export async function get_all_keybindings(): Promise<UserKeybinding[] | null> {
	try {
		const apiUrl = `${API_BASE_URL}/api/command`;

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
 * Calls: PUT /api/command/:id <- TODO: Update endpoint path if necessary (e.g., /api/keybinding/:id)
 */
export async function add_update_keybinding(commandId: number, keybinding: string): Promise<UserKeybinding | null> {
	try {
		const apiUrl = `${API_BASE_URL}/api/command/${commandId}`;

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
 * Calls: DELETE /api/command/:id <- TODO: Update endpoint path if necessary (e.g., /api/keybinding/:id)
 * Updated to return boolean based on previous fixes.
 */
export async function delete_keybinding(commandId: number): Promise<boolean> {
	try {
		const apiUrl = `${API_BASE_URL}/api/command/${commandId}`;

		const response = await fetch(apiUrl, {
			method: 'DELETE',
			credentials: 'include'
		});

		// Simply return true if status is ok (2xx), false otherwise
		return response.ok; 

	} catch (error) {
		console.error('Error deleting keybinding:', error);
		return false; // Return false on network or other errors
	}
}