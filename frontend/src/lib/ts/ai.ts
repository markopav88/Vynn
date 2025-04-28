export interface WritingAssistantSession {
    id: number;
    user_id: number;
    document_id: number | null;
    title: string;
    created_at: string;
    updated_at: string; 
}

export interface WritingAssistantMessage {
    id: number;
    session_id: number;
    role: 'user' | 'assistant' | string;
    content: string;
    created_at: string;
}

export interface SessionWithMessages {
    session: WritingAssistantSession;
    messages: WritingAssistantMessage[];
}

// Payload for creating a new session
export interface CreateSessionPayload {
    document_id: number | null;
    title: string;
}

// Payload for sending a message
export interface SendMessagePayload {
    content: string;
}

// Expected structure for the response from the send_message API
export interface AssistantResponse {
    role: 'assistant';
    content: string;
}

/**
 * Fetches all writing assistant sessions for the current user.
 */
export async function get_all_writing_sessions(): Promise<WritingAssistantSession[]> {
    try {
        const response = await fetch('http://localhost:3001/api/writing-assistant', {
            credentials: 'include'
        });
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        return await response.json();
    } catch (error) {
        console.error('Error fetching writing sessions:', error);
        return [];
    }
}

/**
 * Creates a new writing assistant session.
 */
export async function create_writing_session(payload: CreateSessionPayload): Promise<WritingAssistantSession | null> {
    try {
        const response = await fetch('http://localhost:3001/api/writing-assistant', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
            credentials: 'include'
        });
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        return await response.json();
    } catch (error) {
        console.error('Error creating writing session:', error);
        return null;
    }
}

/**
 * Fetches a specific writing session including its messages.
 */
export async function get_writing_session(sessionId: number): Promise<SessionWithMessages | null> {
    try {
        const response = await fetch(`http://localhost:3001/api/writing-assistant/${sessionId}`, {
            method: 'GET',
            credentials: 'include'
        });
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        return await response.json();
    } catch (error) {
        console.error(`Error fetching writing session ${sessionId}:`, error);
        return null;
    }
}

/**
 * Sends a message to a specific session and gets the AI response.
 */
export async function send_writing_message(sessionId: number, payload: SendMessagePayload): Promise<AssistantResponse | null> {
     try {
        const response = await fetch(`http://localhost:3001/api/writing-assistant/${sessionId}/message`, {
            method: 'POST',
             headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
            credentials: 'include'
        });
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        return await response.json();
    } catch (error) {
        console.error(`Error sending message to session ${sessionId}:`, error);
        return null;
    }
}

/**
 * Deletes a specific writing session.
 */
export async function delete_writing_session(sessionId: number): Promise<boolean> {
    try {
        const response = await fetch(`http://localhost:3001/api/writing-assistant/${sessionId}`, {
            method: 'DELETE',
            credentials: 'include'
        });
        return response.ok;
    } catch (error) {
        console.error(`Error deleting writing session ${sessionId}:`, error);
        return false;
    }
}
