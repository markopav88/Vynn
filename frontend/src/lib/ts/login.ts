/*
/ Login.ts
/
/ File containing functions and logic required for frontend handling of logging in
/ Will provide the communication with the backend and pass necessary information to API calls
/
/ Summary:
/ Class Login: Class responsible for holding login information
/ attempt_login: function responsible for calling POST API for login
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

// Function for attempting login on POST API
export async function attempt_login(login_payload: Login): Promise<boolean> {
    // Use the correct backend API URL
    const apiUrl = `http://localhost:3001/api/login`;

    // attempt to call POST API
    try {
        const response = await fetch(apiUrl, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(login_payload),
            credentials: 'include'
        });

        // Check if the response is successful
        if (response.ok) {
            return true;
        } else {
            console.error('Login failed with status:', response.status);
            const errorText = await response.text();
            console.error('Error response:', errorText);
            return false;
        }
    } catch (error) {
        console.error("Login request error:", error);
        return false;
    }
}

export async function logout() {
    const apiUrl = `http://localhost:3001/api/users/logout`;
    // Call GET API
    const response = await fetch(apiUrl, {
        credentials: 'include'
    });

    // check response status
    if (!response.ok) {
        throw new Error(`Failed to logout: ${response.statusText}`);
    }

    // Check if the response is JSON
    const contentType = response.headers.get('Content-Type');
    if (!contentType || !contentType.includes('application/json')) {
        // If the response is not JSON, log it and return null
        const text = await response.text(); // Read the response as text to inspect it
        console.error('Expected JSON, but received:', text);
        return null;
    }
}