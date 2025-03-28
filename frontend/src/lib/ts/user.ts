/*
/ User.ts
/
/ File containing functions and logic required for frontend handling of users
/ Will provide the communication with the backend and pass necessary information to API calls
/
/ Summary:
/ Class Login: Class responsible for holding login information
/ attempt_login: function responsible for calling POST API for login
/ logout: function responsible for calling GET API for logout
/ get_current_user: function responsible for calling GET API for current user
/ update_user: function responsible for calling PUT API for updating user
/
*/

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
 * Calls: POST /api/login
 */
export async function attempt_login(login_payload: Login): Promise<boolean> {
	const apiUrl = `http://localhost:3001/api/login`;

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
 */
export async function logout(): Promise<boolean> {
	const apiUrl = `http://localhost:3001/api/users/logout`;

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
 * Calls: GET /api/users/:id
 */
export async function get_current_user(): Promise<any | null> {
	try {
		const apiUrl = `http://localhost:3001/api/users/current`;
		
		const response = await fetch(apiUrl, {
			credentials: 'include'
		});
		
		if (!response.ok) {
			console.error('Failed to fetch current user:', response.status);
			return null;
		}
		
		return await response.json();
	} catch (error) {
		console.error('Error fetching current user:', error);
		return null;
	}
}

/**
 * Function to update the current user's information
 * Calls: PUT /api/users/update
 */
export async function update_user(name: string, email: string, password: string): Promise<boolean> {
	try {
		const apiUrl = `http://localhost:3001/api/users/update`;
		
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
    const apiUrl = `http://localhost:3001/api/users`;

    // Attempt to call POST API
    try {
        // Call POST on API
        const response = await fetch(apiUrl, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(signup_payload),
        });

        // Check Response
        if(response.ok) {
            return true;
        } else {
            console.error('Signup failed with status:', response.status);
            const errorText = await response.text();
            console.error('Error response:', errorText);
            return false;
        }
    } catch(error) {
        console.error("Signup request error:", error);
        return false;
    }
} 