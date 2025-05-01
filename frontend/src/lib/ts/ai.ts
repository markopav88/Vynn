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

/**
 * Placeholder function to check grammar.
 * Replace with actual backend API call.
 */
export async function check_grammar(text: string): Promise<string | null> {
	console.log('AI Request: Check grammar for text:', text.substring(0, 100) + '...');
	try {
		return `(Placeholder) Grammar checked for: ${text.substring(0, 50)}...`;
	} catch (error) {
		console.error('Error checking grammar:', error);
		return null;
	}
}

/**
 * Placeholder function to summarize text.
 * Replace with actual backend API call.
 */
export async function summarize_text(text: string): Promise<string | null> {
	console.log('AI Request: Summarize text:', text.substring(0, 100) + '...');
	try {
		return `(Placeholder) Summary of: ${text.substring(0, 50)}...`;
	} catch (error) {
		console.error('Error summarizing text:', error);
		return null;
	}
}

/**
 * Placeholder function to rephrase text.
 * Replace with actual backend API call.
 */
export async function rephrase_text(text: string): Promise<string | null> {
	console.log('AI Request: Rephrase text:', text.substring(0, 100) + '...');
    try {
		return `(Placeholder) Rephrased: ${text.substring(0, 50)}...`;
	} catch (error) {
		console.error('Error rephrasing text:', error);
		return null;
	}
}

/**
 * Placeholder function to expand text.
 * Replace with actual backend API call.
 */
export async function expand_text(text: string): Promise<string | null> {
	console.log('AI Request: Expand text:', text.substring(0, 100) + '...');
	try {
		return `(Placeholder) Expanded: ${text.substring(0, 50)}... plus more details.`;
	} catch (error) {
		console.error('Error expanding text:', error);
		return null;
	}
}

/**
 * Placeholder function to shrink text.
 * Replace with actual backend API call.
 */
export async function shrink_text(text: string): Promise<string | null> {
	console.log('AI Request: Shrink text:', text.substring(0, 100) + '...');
	try {
		return `(Placeholder) Shrunk: ${text.substring(0, 30)}...`;
	} catch (error) {
		console.error('Error shrinking text:', error);
		return null;
	}
}

/**
 * Placeholder function to rewrite text in a specific style.
 * Replace with actual backend API call.
 */
export async function rewrite_text_as(text: string, style: string): Promise<string | null> {
	console.log(`AI Request: Rewrite text as ${style}:`, text.substring(0, 100) + '...');
	try {
		return `(Placeholder) Rewritten as ${style}: ${text.substring(0, 50)}...`;
	} catch (error) {
		console.error(`Error rewriting text as ${style}:`, error);
		return null;
	}
}

/**
 * Placeholder function to fact-check text.
 * Replace with actual backend API call.
 */
export async function fact_check_text(text: string): Promise<string | null> {
	console.log('AI Request: Fact-check text:', text.substring(0, 100) + '...');
	try {
		return `(Placeholder) Fact-checked: ${text.substring(0, 50)}... Looks okay!`;
	} catch (error) {
		console.error('Error fact-checking text:', error);
		return null;
	}
}