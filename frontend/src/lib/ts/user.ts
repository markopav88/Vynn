/*
/ User.ts
/
/ File containing functions and logic required for frontend handling of users
/ Will provide the communication with the backend and pass necessary information to API calls
/
/ Summary:
/ Class User: Mapper of a class to how we are storing users in db
/ Class Login: Data structure for login requests
/ attempt_login: Function to attempt user login with credentials
/ attempt_signup: Function to register a new user
/ logout: Function to log out the current user
/ get_current_user: Function to get the currently logged in user
/ update_user: Function to update user information
/ check_auth: Function to check if a user is authenticated
/
*/

const API_BASE_URL = process.env.API_BASE_URL;

export interface User {
	id: number;
	name: string;
	email: string;
	ai_credits?: number;
}

// Class for holding login information
export class Login {
	email: string;
	password: string;

	constructor(new_email: string, new_password: string) {
		this.email = new_email;
		this.password = new_password;
	}
}

// Class for holding signup information from frontend
export class Signup {
	name: string;
	email: string;
	password_one: string;
	password_two: string;

	constructor(new_name: string, new_email: string, new_password_one: string, new_password_two: string) {
		this.name = new_name;
		this.email = new_email;
		this.password_one = new_password_one;
		this.password_two = new_password_two;
	}
}

// Class for holding converted payload for backend
class SignupPayload {
	name: string;
	email: string;
	password: string;

	constructor(new_name: string, new_email: string, new_password: string) {
		this.name = new_name;
		this.email = new_email;
		this.password = new_password;
	}
}

/**
 * Function to attempt login
 * Calls: POST /api/users/login
 * Test: test_users.rs/test_good_login()
 */
export async function attempt_login(login_payload: Login): Promise<boolean> {
	const apiUrl = `${API_BASE_URL}/api/users/login`;

	try {
		const response = await fetch(apiUrl, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(login_payload),
			credentials: 'include'
		});

		if (response.ok) {
			return true;
		} else {
			console.error('Login failed with status:', response.status);
			const errorText = await response.text();
			console.error('Error response:', errorText);
			return false;
		}
	} catch (error) {
		console.error('Login request error:', error);
		return false;
	}
}

/**
 * Function to logout user
 * Calls: GET /api/users/logout
 * Test: test_users.rs/test_logout()
 */
export async function logout(): Promise<boolean> {
	const apiUrl = `${API_BASE_URL}/api/users/logout`;

	try {
		const response = await fetch(apiUrl, {
			credentials: 'include'
		});

		if (!response.ok) {
			throw new Error(`Failed to logout: ${response.statusText}`);
		}

		return true;
	} catch (error) {
		console.error('Logout error:', error);
		return false;
	}
}

/**
 * Function to get the current user's information
 * Calls: GET /api/users/current
 * Test: test_users.rs/test_get_current_user()
 */
export async function get_current_user(): Promise<User | null> {
	try {
		const apiUrl = `${API_BASE_URL}/api/users/current`;

		const response = await fetch(apiUrl, {
			credentials: 'include'
		});

		if (!response.ok) {
			console.error('Failed to fetch current user:', response.status);
			return null;
		}

		const user: User = await response.json();
		console.log('Fetched current user:', user);
		return user;
	} catch (error) {
		console.error('Error fetching current user:', error);
		return null;
	}
}

/**
 * Function to update the current user's information
 * Calls: PUT /api/users/update
 * Test: test_users.rs/test_update_user()
 */
export async function update_user(name: string, email: string, password: string): Promise<boolean> {
	try {
		const apiUrl = `${API_BASE_URL}/api/users/update`;

		const payload = {
			name: name,
			email: email,
			password: password
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
		console.error('Error updating user:', error);
		return false;
	}
}

/**
 * Function for attempting signup
 * Calls: POST /api/users
 * Test: test_users.rs/test_create_user()
 */
export async function attempt_signup(signup_input: Signup): Promise<boolean> {
	// Check if passwords match and make new payload if they do
	let signup_payload;
	if (signup_input.password_one != signup_input.password_two) {
		return false;
	} else {
		signup_payload = new SignupPayload(signup_input.name, signup_input.email, signup_input.password_one);
	}

	// Use the correct backend API URL
	const apiUrl = `${API_BASE_URL}/api/users`;

	// Attempt to call POST API
	try {
		// Call POST on API
		const response = await fetch(apiUrl, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(signup_payload)
		});

		// Check Response
		if (response.ok) {
			return true;
		} else {
			console.error('Signup failed with status:', response.status);
			const errorText = await response.text();
			console.error('Error response:', errorText);
			return false;
		}
	} catch (error) {
		console.error('Signup request error:', error);
		return false;
	}
}

/**
 * Check if the user is authenticated
 * Calls: GET /api/users/check-auth
 * Test: TODO: test_users.rs/test_check_auth() - Test missing
 * @returns Promise<boolean> - True if authenticated, false otherwise
 */
export async function check_auth(): Promise<boolean> {
	try {
		// Add fallback logic in case the backend is not available
		const apiUrl = `${API_BASE_URL}/api/users/check-auth`;
		console.log('Attempting to connect to backend at:', apiUrl);

		// Add timeout to prevent long waiting times if server is down
		const controller = new AbortController();
		const timeoutId = setTimeout(() => controller.abort(), 3000);

		const response = await fetch(apiUrl, {
			method: 'GET',
			credentials: 'include',
			signal: controller.signal
		});

		clearTimeout(timeoutId);

		if (response.ok) {
			const data = await response.json();
			console.log('Auth check response:', data);
			return data.authenticated;
		} else {
			console.error('Auth check failed with status:', response.status);
			return false;
		}
	} catch (error) {
		console.error('Auth check request error:', error);
		// Continue with the app even if backend is not available
		return false;
	}
}

/**
 * Function to upload a profile image
 * Calls: POST /api/users/profile-image
 * Test: TODO: test_users.rs/test_upload_profile_image() - Test missing
 */
export async function upload_profile_image(file: File): Promise<boolean> {
	const apiUrl = `${API_BASE_URL}/api/users/profile-image`;
	const formData = new FormData();
	formData.append('profile_image', file);

	try {
		const response = await fetch(apiUrl, {
			method: 'POST',
			body: formData,
			credentials: 'include'
		});

		if (!response.ok) {
			console.error('Failed to upload profile image:', response.status);
			return false;
		}

		return true;
	} catch (error) {
		console.error('Error uploading profile image:', error);
		return false;
	}
}

/**
 * Function to get user's profile image URL
 * Note: This generates the URL, doesn't call a specific check endpoint itself.
 * Relies on backend route: GET /api/users/:id/profile-image
 * Test: TODO: test_users.rs/test_get_profile_image() - Test missing
 * @returns string - URL that can be used in img src attribute
 */
export function get_profile_image_url(userId: number): string {
	return `${API_BASE_URL}/api/users/${userId}/profile-image?t=${new Date().getTime()}`;
}
