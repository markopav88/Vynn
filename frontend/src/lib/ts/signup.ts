/*
/ Signup.ts
/
/ File containing functions and logic required for frontend handling of signing up
/ Will provide the communication with the backend and pass necessary information to API calls
/
/ Summary:
/ Class Signup: Class responsible for holding signup information from frontend
/ Class SignupPayload: Class responsible for holding signup information to backend
/ attempt_signup: Function responsible for handling logic, checks, and sending POST API to /users
/
*/

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

// Function for attempting signup on POST API
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
            console.error('Login failed with status:', response.status);
            const errorText = await response.text();
            console.error('Error response:', errorText);
            return false;
        }
    } catch(error) {
        console.error("Login request error:", error);
        return false;
    }
}