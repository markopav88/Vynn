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

// Define Payload structure for AI text commands
interface AiTextPayload {
	content: string;
}

// Define Payload for rewrite command
interface AiRewritePayload {
	content: string;
	style: string;
}

// Define expected Response structure
interface AiCommandResponse {
	response: string;
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
 * Function to check grammar using the backend API.
 */
export async function check_grammar(text: string): Promise<AiCommandResponse | null> {
	console.log('AI Request: Check grammar for text:', text.substring(0, 100) + '...');
	const payload: AiTextPayload = { content: text };
	try {
		const response = await fetch('http://localhost:3001/api/writing-assistant/grammer', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(payload),
			credentials: 'include'
		});
		if (!response.ok) {
			throw new Error(`HTTP error! status: ${response.status}`);
		}
		const result: AiCommandResponse = await response.json();
		console.log('Grammar Check Response:', result.response);
		return result;
	} catch (error) {
		console.error('Error checking grammar:', error);
		return null;
	}
}

/**
 * Function to summarize text using the backend API.
 */
export async function summarize_text(text: string): Promise<AiCommandResponse | null> {
	console.log('AI Request: Summarize text:', text.substring(0, 100) + '...');
	const payload: AiTextPayload = { content: text };
	try {
		const response = await fetch('http://localhost:3001/api/writing-assistant/summarize', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(payload),
			credentials: 'include'
		});
		if (!response.ok) {
			throw new Error(`HTTP error! status: ${response.status}`);
		}
		const result: AiCommandResponse = await response.json();
		console.log('Summarize Response:', result.response);
		return result;
	} catch (error) {
		console.error('Error summarizing text:', error);
		return null;
	}
}

/**
 * Function to rephrase text using the backend API.
 */
export async function rephrase_text(text: string): Promise<AiCommandResponse | null> {
	console.log('AI Request: Rephrase text:', text.substring(0, 100) + '...');
    const payload: AiTextPayload = { content: text };
	try {
		const response = await fetch('http://localhost:3001/api/writing-assistant/rephrase', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(payload),
			credentials: 'include'
		});
		if (!response.ok) {
			throw new Error(`HTTP error! status: ${response.status}`);
		}
		const result: AiCommandResponse = await response.json();
		console.log('Rephrase Response:', result.response);
		return result;
	} catch (error) {
		console.error('Error rephrasing text:', error);
		return null;
	}
}

/**
 * Function to expand text using the backend API.
 */
export async function expand_text(text: string): Promise<AiCommandResponse | null> {
	console.log('AI Request: Expand text:', text.substring(0, 100) + '...');
	const payload: AiTextPayload = { content: text };
	try {
		const response = await fetch('http://localhost:3001/api/writing-assistant/expand', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(payload),
			credentials: 'include'
		});
		if (!response.ok) {
			throw new Error(`HTTP error! status: ${response.status}`);
		}
		const result: AiCommandResponse = await response.json();
		console.log('Expand Response:', result.response);
		return result;
	} catch (error) {
		console.error('Error expanding text:', error);
		return null;
	}
}

/**
 * Function to shrink text using the backend API.
 */
export async function shrink_text(text: string): Promise<AiCommandResponse | null> {
	console.log('AI Request: Shrink text:', text.substring(0, 100) + '...');
	const payload: AiTextPayload = { content: text };
	try {
		const response = await fetch('http://localhost:3001/api/writing-assistant/shrink', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(payload),
			credentials: 'include'
		});
		if (!response.ok) {
			throw new Error(`HTTP error! status: ${response.status}`);
		}
		const result: AiCommandResponse = await response.json();
		console.log('Shrink Response:', result.response);
		return result;
	} catch (error) {
		console.error('Error shrinking text:', error);
		return null;
	}
}

/**
 * Function to rewrite text in a specific style using the backend API.
 */
export async function rewrite_text_as(text: string, style: string): Promise<AiCommandResponse | null> {
	console.log(`AI Request: Rewrite text as ${style}:`, text.substring(0, 100) + '...');
	const payload: AiRewritePayload = { content: text, style };
	try {
		const response = await fetch('http://localhost:3001/api/writing-assistant/rewrite', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(payload),
			credentials: 'include'
		});
		if (!response.ok) {
			throw new Error(`HTTP error! status: ${response.status}`);
		}
		const result: AiCommandResponse = await response.json();
		console.log('Rewrite Response:', result.response);
		return result;
	} catch (error) {
		console.error(`Error rewriting text as ${style}:`, error);
		return null;
	}
}

/**
 * Placeholder function to fact-check text.
 * This should ideally call a dedicated fact-checking API, not just the LLM.
 */
export async function fact_check_text(text: string): Promise<AiCommandResponse | null> {
	console.warn('Fact-checking called with text:', text.substring(0, 100) + '...');
	const payload: AiTextPayload = { content: text };
	try {
		// Call the backend API endpoint for fact-checking
		const response = await fetch('http://localhost:3001/api/writing-assistant/factcheck', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(payload),
			credentials: 'include'
		});

		if (!response.ok) {
			throw new Error(`HTTP error! status: ${response.status}`);
		}

		const result: AiCommandResponse = await response.json();
		console.log('Fact Check Response:', result.response);
		return result;
	} catch (error) {
		console.error('Error during fact-checking:', error);
		return null;
	}
}