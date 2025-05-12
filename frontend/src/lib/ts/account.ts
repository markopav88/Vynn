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

export interface Preference {
    preference_id: number;
    preference_name: string;
    preference_value: string;
    preference_description: string;
}

/**
 * Function to get all user preferences
 * Calls: GET /api/preference
 */
export async function get_all_preferences(): Promise<Preference[] | null> {
    try {
        const apiUrl = `${API_BASE_URL}/api/preference`;

        const response = await fetch(apiUrl, {
            credentials: 'include',
            headers: {
                'Content-Type': 'application/json'
            }
        });

        if (!response.ok) {
            console.error('Failed to fetch preferences:', response.status);
            return null;
        }

        return await response.json();
    } catch (error) {
        console.error('Error fetching preferences:', error);
        return null;
    }
}

/**
 * Function to update a preference
 * Calls: PUT /api/preference/:id
 */
export async function update_preference(preferenceId: number, value: string): Promise<boolean> {
    try {
        const apiUrl = `${API_BASE_URL}/api/preference/${preferenceId}`;

        const response = await fetch(apiUrl, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ preference_value: value }),
            credentials: 'include'
        });

        return response.ok;
    } catch (error) {
        console.error('Error updating preference:', error);
        return false;
    }
}

/**
 * Function to reset a preference to default
 * Calls: DELETE /api/preference/:id
 */
export async function reset_preference(preferenceId: number): Promise<boolean> {
    try {
        const apiUrl = `${API_BASE_URL}/api/preference/${preferenceId}`;

        const response = await fetch(apiUrl, {
            method: 'DELETE',
            credentials: 'include'
        });

        return response.ok;
    } catch (error) {
        console.error('Error resetting preference:', error);
        return false;
    }
}

/**
 * Function to reset all preferences to default
 * Calls: DELETE /api/preference
 */
export async function reset_all_preferences(): Promise<boolean> {
    try {
        const apiUrl = `${API_BASE_URL}/api/preference`;

        const response = await fetch(apiUrl, {
            method: 'DELETE',
            credentials: 'include'
        });

        return response.ok;
    } catch (error) {
        console.error('Error resetting all preferences:', error);
        return false;
    }
}

/**
 * Function to check if background image exists
 * Calls: GET /api/preference/background
 */
export async function check_background_image(): Promise<{ imageUrl: string | null, isCustom: boolean }> {
    try {
        const timestamp = new Date().getTime();
        const apiUrl = `${API_BASE_URL}/api/preference/background?t=${timestamp}`;
        console.log(`Checking background image at: ${apiUrl}`); // Debug log

        const response = await fetch(apiUrl, {
            method: 'GET',
            credentials: 'include'
        });

        if (response.ok) {
            const blob = await response.blob(); // Get the image as a blob
            
            // Check if the blob size is greater than 0
            if (blob.size === 0) {
                console.log('Received an empty image blob. Fetching default image.'); // Debug log
                
                // Fetch the default background image
                const defaultImageResponse = await fetch(`${API_BASE_URL}/api/preference/default-background`, {
                    method: 'GET',
                    credentials: 'include'
                });

                if (defaultImageResponse.ok) {
                    const defaultBlob = await defaultImageResponse.blob();
                    const defaultImageUrl = URL.createObjectURL(defaultBlob);
                    return { imageUrl: defaultImageUrl, isCustom: false }; // Return the default image URL
                } else {
                    console.log('Failed to fetch default background image.'); // Debug log
                    return { imageUrl: null, isCustom: false }; // Return null if default image fetch fails
                }
            }

            const imageUrl = URL.createObjectURL(blob); // Create a URL for the blob
            console.log('Background image found, returning URL.'); // Debug log
            return { imageUrl, isCustom: true }; // Return the blob URL and indicate it's custom
        } else {
            console.log('No background image found, fetching default image.'); // Debug log
            // Fetch the default background image
            const defaultImageResponse = await fetch(`${API_BASE_URL}/api/preference/default-background`, {
                method: 'GET',
                credentials: 'include'
            });

            if (defaultImageResponse.ok) {
                const defaultBlob = await defaultImageResponse.blob();
                const defaultImageUrl = URL.createObjectURL(defaultBlob);
                return { imageUrl: defaultImageUrl, isCustom: false }; // Return the default image URL
            }

            return { imageUrl: null, isCustom: false }; // Return null if default image fetch fails
        }
    } catch (error) {
        console.error('Error checking background image:', error);
        return { imageUrl: null, isCustom: false }; // Return null and indicate it's not custom
    }
}

/**
 * Function to upload background image
 * Calls: POST /api/preference/background
 */
export async function upload_background_image(file: File): Promise<boolean> {
    try {
        const apiUrl = `${API_BASE_URL}/api/preference/background`;
        const formData = new FormData();
        formData.append('background_image', file);

        const response = await fetch(apiUrl, {
            method: 'POST',
            body: formData,
            credentials: 'include'
        });

        return response.ok;
    } catch (error) {
        console.error('Error uploading background image:', error);
        return false;
    }
}

/**
 * Function to reset background image
 * Calls: DELETE /api/preference/background
 */
export async function reset_background_image(): Promise<boolean> {
    try {
        const apiUrl = `${API_BASE_URL}/api/preference/background`;

        const response = await fetch(apiUrl, {
            method: 'DELETE',
            credentials: 'include'
        });

        return response.ok;
    } catch (error) {
        console.error('Error resetting background image:', error);
        return false;
    }
}

// Function to convert hex to RGBA
export function hexToRgba(hex: string, alpha: number = 1): string {
    hex = hex.replace(/^#/, '');

    let r: number, g: number, b: number;

    if (hex.length === 3) {
        r = parseInt(hex[0] + hex[0], 16);
        g = parseInt(hex[1] + hex[1], 16);
        b = parseInt(hex[2] + hex[2], 16);
    } else if (hex.length === 6) {
        r = parseInt(hex.slice(0, 2), 16);
        g = parseInt(hex.slice(2, 4), 16);
        b = parseInt(hex.slice(4, 6), 16);
    } else {
        throw new Error('Invalid hex color format');
    }

    return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}