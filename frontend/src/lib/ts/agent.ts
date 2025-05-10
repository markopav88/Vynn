/*
/ agent.ts
/
/ This file contains the logic for the AI agent that decides whether to proactively
/ show a diff view to the user based on the AI's response and the current context.
/ It defines the necessary data structures and makes an API call to a decision-making
/ AI model in the backend.
/
/ Summary:
/ Interfaces:
/ - ProactiveDiffContext: Defines the structure of the context passed to the agent,
/                         including the type of interaction (colon command or chat)
/                         and relevant details like command name or user prompt.
/ 
/ Functions:
/ - shouldAgentShowDiffProactively: The main exported function that determines if a diff
/                                   should be shown. It calls a backend service to get
/                                   this decision.
/ - fetchProactiveDiffDecisionFromAPI: (Internal) Handles the actual API call to the
/                                        backend endpoint that provides the True/False decision.
*/

/**
 * Defines the context for the proactive diff decision.
 * This information is sent to the backend to help the AI decide if a diff is appropriate.
 */
export interface ProactiveDiffContext {
    type: 'colonCommand' | 'chat';
    commandName?: string; // e.g., 'grammar', 'summarize'
    userPrompt?: string;  // The user's message that triggered the AI
}

// Payload for the new sanitize-text endpoint
interface SanitizeTextPayload {
    text_to_sanitize: string;
}

// Response for the new sanitize-text endpoint
interface SanitizeTextResponse {
    sanitized_text: string;
}

// Function to fetch the proactive diff decision from the API
// This mirrors the logic that was in ai.ts's getProactiveDiffDecision
const API_BASE_URL = process.env.API_BASE_URL; // Ensure this is accessible

/**
 * (Internal function)
 * Fetches the AI's decision on whether to show a diff proactively.
 * Calls: POST /api/ai/decide-proactive-diff
 * @param aiResponseContent The content of the AI's response.
 * @param context The ProactiveDiffContext providing details about the user's interaction.
 * @param documentContent The actual current content of the document (or a snippet/empty string).
 * @returns A promise that resolves to "True" or "False" as a string.
 */
async function fetchProactiveDiffDecisionFromAPI(
    aiResponseContent: string,
    context: ProactiveDiffContext,
    documentContent: string, // New direct parameter
): Promise<string> {
    const apiUrl = `${API_BASE_URL}/api/writing-assistant/decide-proactive-diff`;
    const payload = {
        aiResponseContent,
        context: { 
            type: context.type,
            commandName: context.commandName,
            userPrompt: context.userPrompt
        },
        documentContentSnippet: documentContent
    };
    console.log("[Agent API Call] fetchProactiveDiffDecisionFromAPI payload:", payload);
    try {
        const response = await fetch(apiUrl, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
            credentials: 'include'
        });

        if (!response.ok) {
            let errorBody;
            try {
                errorBody = await response.json();
            } catch (e) {
                // Ignore if response body is not JSON
            }
            console.error(`[Agent API Call] fetchProactiveDiffDecisionFromAPI failed: ${response.status} ${response.statusText}`, errorBody);
            throw new Error(`HTTP error ${response.status}: ${errorBody?.error?.message || response.statusText}`);
        }
        
        const result = await response.json();
        console.log("[Agent API Call] fetchProactiveDiffDecisionFromAPI response:", result);
        return result.decision; 
    } catch (error) {
        console.error("[Agent API Call] Error in fetchProactiveDiffDecisionFromAPI:", error);
        if (error instanceof Error && error.message.startsWith('HTTP error')) {
            throw error;
        }
        return 'False'; 
    }
}

/**
 * Determines whether an AI-suggested diff should be shown proactively to the user.
 * This function calls a backend AI service to get a True/False decision based on the
 * AI's response content, the context of the user's interaction, and the current document content.
 *
 * @param aiResponseContent The full content of the AI's response message.
 * @param context An object providing context about the user's action that triggered the AI.
 * @param currentDocumentContent The current textual content of the document.
 * @returns A promise that resolves to a boolean: true if a diff should be shown, false otherwise.
 */
export async function shouldAgentShowDiffProactively(
    aiResponseContent: string,
    context: ProactiveDiffContext,
    currentDocumentContent: string,
): Promise<boolean> {
    console.log('[Agent] shouldAgentShowDiffProactively called. Context:', context, "AI Response snippet:", aiResponseContent.substring(0,50), "Doc content snippet:", currentDocumentContent.substring(0,50));
    try {
        const decisionString = await fetchProactiveDiffDecisionFromAPI(aiResponseContent, context, currentDocumentContent);
        console.log("[Agent] Received decision string from API:", decisionString);
        const decision = decisionString.trim().toLowerCase() === 'true';
        console.log(`[Agent] Final decision on proactive diff: ${decision}`);
        return decision;
    } catch (error) {
        console.error("[Agent] Error in shouldAgentShowDiffProactively during API call:", error);
        return false; 
    }
} 

/**
 * Sends text to the backend for HTML and Markdown sanitization.
 * Calls: POST /api/ai/writing-assistant/sanitize-text
 */
export async function sanitizeText(textToSanitize: string): Promise<string> {
    console.log('[Agent - sanitizeText] Sending text for sanitization:', textToSanitize.substring(0, 70) + '...');
    const payload: SanitizeTextPayload = { text_to_sanitize: textToSanitize };
    const apiUrl = `${API_BASE_URL}/api/writing-assistant/sanitize-text`; 

    try {
        const response = await fetch(apiUrl, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
            credentials: 'include'
        });

        if (!response.ok) {
            let errorBody;
            try {
                errorBody = await response.json();
            } catch (e) {
                // Ignore if response body is not JSON
            }
            console.error(`[Agent - sanitizeText] API call failed: ${response.status} ${response.statusText}`, errorBody);
            // Throw an error that can be caught by the caller, or return original text
            throw new Error(`HTTP error ${response.status}: ${errorBody?.error?.message || response.statusText}`);
        }
        
        const result: SanitizeTextResponse = await response.json();
        console.log('[Agent - sanitizeText] Sanitized text received:', result.sanitized_text.substring(0, 70) + '...');
        return result.sanitized_text;
    } catch (error) {
        console.error('[Agent - sanitizeText] Error sanitizing text:', error);
        if (error instanceof Error && error.message.startsWith('HTTP error')) {
            throw error; 
        }
        return textToSanitize; 
    }
}