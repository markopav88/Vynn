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
        });

        // Check if the response is successful (status code 200-299)
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