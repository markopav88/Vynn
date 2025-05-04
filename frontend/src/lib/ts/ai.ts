/*
/ ai.ts
/
/ File containing functions and logic required for frontend handling of the AI Writing Assistant features.
/ Provides communication with the backend for AI-related operations like managing sessions, 
/ sending messages, and executing specific AI text commands (grammar check, summarize, etc.).
/
/ Summary:
/ Interfaces:
/ - WritingAssistantSession: Represents a writing assistant session.
/ - WritingAssistantMessage: Represents a message within a session.
/ - SessionWithMessages: Combines session data with its messages.
/ - CreateSessionPayload: Payload for creating a new session.
/ - SendMessagePayload: Payload for sending a message to a session.
/ - AssistantResponse: Expected structure for an AI assistant's response message.
/ - AiTextPayload: Generic payload for AI commands taking text content.
/ - AiRewritePayload: Payload for the rewrite command, including style.
/ - AiCommandResponse: Expected structure for responses from AI text commands.
/ 
/ Functions:
/ - get_all_writing_sessions: Fetches all sessions for the current user.
/ - create_writing_session: Creates a new writing session.
/ - get_writing_session: Fetches a specific session and its messages.
/ - send_writing_message: Sends a message to a session and gets the AI response.
/ - delete_writing_session: Deletes a specific writing session.
/ - check_grammar: Sends text to the backend for grammar checking.
/ - summarize_text: Sends text to the backend for summarization.
/ - rephrase_text: Sends text to the backend for rephrasing.
/ - expand_text: Sends text to the backend for expansion.
/ - shrink_text: Sends text to the backend for shrinking.
/ - rewrite_text_as: Sends text and a style to the backend for rewriting.
/ - fact_check_text: Sends text to the backend for fact-checking.
/ - check_spelling: Sends text to the backend for spell checking.
/
*/

const API_BASE_URL = process.env.API_BASE_URL;

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
 * Calls: GET /api/writing-assistant
 * Test: test_ai.rs/test_get_all_writing_sessions_success()
 */
export async function get_all_writing_sessions(): Promise<WritingAssistantSession[] | null> {
    const apiUrl = `${API_BASE_URL}/api/writing-assistant`;
    try {
        const response = await fetch(apiUrl, { 
            credentials: 'include'
        });
        if (!response.ok) {
            console.error('Failed get sessions:', response.status, response.statusText);
			throw response;
        }
        return await response.json();
    } catch (error) {
        console.error('Error fetching writing sessions:', error);
        throw error;
    }
}

/**
 * Creates a new writing assistant session.
 * Calls: POST /api/writing-assistant
 * Test: test_ai.rs/test_create_writing_session_success()
 */
export async function create_writing_session(payload: CreateSessionPayload): Promise<WritingAssistantSession | null> {
    const apiUrl = `${API_BASE_URL}/api/writing-assistant`;
    try {
        const response = await fetch(apiUrl, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
            credentials: 'include'
        });
        if (!response.ok) {
            console.error('Create session failed:', response.status, response.statusText);
            throw response;
        }
        return await response.json();
    } catch (error) {
        console.error('Error creating writing session:', error);
        throw error;
    }
}

/**
 * Fetches a specific writing session including its messages.
 * Calls: GET /api/writing-assistant/:sessionId
 * Test: test_ai.rs/test_get_writing_session_success()
 */
export async function get_writing_session(sessionId: number): Promise<SessionWithMessages | null> {
    const apiUrl = `${API_BASE_URL}/api/writing-assistant/${sessionId}`;
    try {
        const response = await fetch(apiUrl, {
            method: 'GET',
            credentials: 'include'
        });
        if (!response.ok) {
            console.error('Get session failed:', response.status, response.statusText);
            throw response;
        }
        return await response.json();
    } catch (error) {
        console.error(`Error fetching writing session ${sessionId}:`, error);
        throw error;
    }
}

/**
 * Sends a message to a specific session and gets the AI response.
 * Calls: POST /api/writing-assistant/:sessionId/message
 * Test: test_ai.rs/test_send_writing_message_success()
 */
export async function send_writing_message(sessionId: number, payload: SendMessagePayload): Promise<AssistantResponse | null> {
     const apiUrl = `${API_BASE_URL}/api/writing-assistant/${sessionId}/message`;
     try {
        const response = await fetch(apiUrl, {
            method: 'POST',
             headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
            credentials: 'include'
        });
        if (!response.ok) {
            console.error('Send message failed:', response.status, response.statusText);
            throw response;
        }
        return await response.json();
    } catch (error) {
        console.error(`Error sending message to session ${sessionId}:`, error);
        throw error;
    }
}

/**
 * Deletes a specific writing session.
 * Calls: DELETE /api/writing-assistant/:sessionId
 * Test: test_ai.rs/test_delete_writing_session_success()
 */
export async function delete_writing_session(sessionId: number): Promise<boolean> {
    const apiUrl = `${API_BASE_URL}/api/writing-assistant/${sessionId}`;
    try {
        const response = await fetch(apiUrl, {
            method: 'DELETE',
            credentials: 'include'
        });
        if (!response.ok) {
            console.error('Delete session failed:', response.status, response.statusText);
            throw response;
        }
        // For DELETE, success is usually just status 2xx
        return true; 
    } catch (error) {
        console.error(`Error deleting writing session ${sessionId}:`, error);
        // If the error is the Response object itself, return false, otherwise rethrow
        if (error instanceof Response) {
            return false;
        }
        throw error; 
    }
}

/**
 * Function to check grammar using the backend API.
 * Calls: POST /api/writing-assistant/grammer
 * Test: test_ai.rs/test_check_grammar_success()
 */
export async function check_grammar(content: string): Promise<AiCommandResponse | null> {
    console.log('Checking grammar with text:', content.substring(0, 50) + '...');
    const payload: AiTextPayload = { content };
    const apiUrl = `${API_BASE_URL}/api/writing-assistant/grammer`;
    try {
        const response = await fetch(apiUrl, { 
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
            credentials: 'include'
        });
        if (!response.ok) {
            console.error('Grammar check failed:', response.status, response.statusText);
            throw response; 
        }
        const result: AiCommandResponse = await response.json();
        console.log('Grammar Check Response:', result.response);
        return result;
    } catch (error) {
        console.error('Error checking grammar:', error);
        throw error; 
    }
}

/**
 * Function to summarize text using the backend API.
 * Calls: POST /api/writing-assistant/summarize
 * Test: test_ai.rs/test_summarize_success()
 */
export async function summarize_text(content: string): Promise<AiCommandResponse | null> {
    console.log('Summarizing text:', content.substring(0, 50) + '...');
    const payload: AiTextPayload = { content };
    const apiUrl = `${API_BASE_URL}/api/writing-assistant/summarize`;
    try {
        const response = await fetch(apiUrl, { 
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
            credentials: 'include'
        });
        if (!response.ok) {
             console.error('Summarize failed:', response.status, response.statusText);
            throw response; 
        }
        const result: AiCommandResponse = await response.json();
        console.log('Summarize Response:', result.response);
        return result;
    } catch (error) {
        console.error('Error summarizing text:', error);
        throw error; 
    }
}

/**
 * Function to rephrase text using the backend API.
 * Calls: POST /api/writing-assistant/rephrase
 * Test: test_ai.rs/test_rephrase_success()
 */
export async function rephrase_text(content: string): Promise<AiCommandResponse | null> {
    console.log('Rephrasing text:', content.substring(0, 50) + '...');
    const payload: AiTextPayload = { content };
    const apiUrl = `${API_BASE_URL}/api/writing-assistant/rephrase`;
    try {
        const response = await fetch(apiUrl, { 
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
            credentials: 'include'
        });
        if (!response.ok) {
            console.error('Rephrase failed:', response.status, response.statusText);
            throw response; 
        }
        const result: AiCommandResponse = await response.json();
        console.log('Rephrase Response:', result.response);
        return result;
    } catch (error) {
        console.error('Error rephrasing text:', error);
        throw error; 
    }
}

/**
 * Function to expand text using the backend API.
 * Calls: POST /api/writing-assistant/expand
 * Test: test_ai.rs/test_expand_success()
 */
export async function expand_text(content: string): Promise<AiCommandResponse | null> {
    console.log('Expanding text:', content.substring(0, 50) + '...');
    const payload: AiTextPayload = { content };
    const apiUrl = `${API_BASE_URL}/api/writing-assistant/expand`;
    try {
        const response = await fetch(apiUrl, { 
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
            credentials: 'include'
        });
        if (!response.ok) {
            console.error('Expand failed:', response.status, response.statusText);
            throw response; 
        }
        const result: AiCommandResponse = await response.json();
        console.log('Expand Response:', result.response);
        return result;
    } catch (error) {
        console.error('Error expanding text:', error);
        throw error; 
    }
}

/**
 * Function to shrink text using the backend API.
 * Calls: POST /api/writing-assistant/shrink
 * Test: test_ai.rs/test_shrink_success()
 */
export async function shrink_text(content: string): Promise<AiCommandResponse | null> {
    console.log('Shrinking text:', content.substring(0, 50) + '...');
    const payload: AiTextPayload = { content };
    const apiUrl = `${API_BASE_URL}/api/writing-assistant/shrink`;
    try {
        const response = await fetch(apiUrl, { 
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
            credentials: 'include'
        });
        if (!response.ok) {
            console.error('Shrink failed:', response.status, response.statusText);
            throw response; 
        }
        const result: AiCommandResponse = await response.json();
        console.log('Shrink Response:', result.response);
        return result;
    } catch (error) {
        console.error('Error shrinking text:', error);
        throw error; 
    }
}

/**
 * Function to rewrite text in a specific style using the backend API.
 * Calls: POST /api/writing-assistant/rewrite
 * Test: test_ai.rs/test_rewrite_success()
 */
export async function rewrite_text_as(content: string, style: string): Promise<AiCommandResponse | null> {
    console.log(`Rewriting text as ${style}:`, content.substring(0, 50) + '...');
    const payload: AiRewritePayload = { content, style };
    const apiUrl = `${API_BASE_URL}/api/writing-assistant/rewrite`;
    try {
        const response = await fetch(apiUrl, { 
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
            credentials: 'include'
        });
        if (!response.ok) {
            console.error('Rewrite failed:', response.status, response.statusText);
            throw response; 
        }
        const result: AiCommandResponse = await response.json();
        console.log('Rewrite Response:', result.response);
        return result;
    } catch (error) {
        console.error('Error rewriting text as:', error);
        throw error; 
    }
}

/**
 * Placeholder function to fact-check text.
 * Calls: POST /api/writing-assistant/factcheck
 * Test: test_ai.rs/test_fact_check_success()
 * This should ideally call a dedicated fact-checking API, not just the LLM.
 */
export async function fact_check_text(content: string): Promise<AiCommandResponse | null> {
    console.log('Fact-checking called with text:', content.substring(0, 50) + '...'); 
    const payload: AiTextPayload = { content };
    const apiUrl = `${API_BASE_URL}/api/writing-assistant/factcheck`;
    try {
        const response = await fetch(apiUrl, { 
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
            credentials: 'include'
        });
        if (!response.ok) {
            console.error('Fact check failed:', response.status, response.statusText);
            throw response;
        }
        const result: AiCommandResponse = await response.json();
        console.log('Fact Check Response:', result.response);
        return result;
    } catch (error) {
        console.error('Error during fact_check_text:', error);
        throw error;
    }
}

/**
 * Function to check spelling using the backend API.
 * Calls: POST /api/writing-assistant/spellcheck
 * Test: test_ai.rs/test_spell_check_success()
 */
export async function check_spelling(content: string): Promise<AiCommandResponse | null> {
    console.log('Checking spelling with text:', content.substring(0, 50) + '...');
    const payload: AiTextPayload = { content };
    const apiUrl = `${API_BASE_URL}/api/writing-assistant/spellcheck`;
    try {
        const response = await fetch(apiUrl, { 
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
            credentials: 'include'
        });
        if (!response.ok) {
            console.error('Spell check failed:', response.status, response.statusText);
            throw response; 
        }
        const result: AiCommandResponse = await response.json();
        console.log('Spell Check Response:', result.response);
        return result;
    } catch (error) {
        console.error('Error checking spelling:', error);
        throw error; 
    }
}