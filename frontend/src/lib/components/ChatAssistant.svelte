<script lang="ts">
    import { onMount, createEventDispatcher } from 'svelte';
    import { get_all_writing_sessions, create_writing_session, get_writing_session, send_writing_message, delete_writing_session, apply_ai_suggestion } from '$lib/ts/ai';
    import type { WritingAssistantSession, WritingAssistantMessage, SessionWithMessages, CreateSessionPayload, SendMessagePayload, SuggestedDocumentChange } from '$lib/ts/ai';
    import { shouldAgentShowDiffProactively } from '$lib/ts/agent';
    import { get_all_preferences, check_background_image } from '$lib/ts/account'; // Adjust imports as necessary

    export let documentId: number | null = null;
    export let isOpen = false;

    const dispatch = createEventDispatcher();

    // Add missing variable declarations here
    let sessions: WritingAssistantSession[] = [];
    let messages: WritingAssistantMessage[] = [];
    let currentSessionId: number | null = null;
    let newMessageContent: string = '';
    let isLoadingSessions: boolean = false;
    let isLoadingMessages: boolean = false;
    let errorLoadingSessions: string | null = null;
    let errorLoadingMessages: string | null = null;
    export let messageInput: HTMLInputElement | null;
    let chatBody: HTMLDivElement;
    let isApplyingSuggestion = false;

    // Preferences variables
    let preferences: any[] = [];
    let primaryColorPref = '#0A1721';
    let secondaryColorPref = '#10b981';
    let primaryAccentColorPref = '#10b981';
    let secondaryAccentColorPref = '#808080';
    let primaryTextColorPref = '#10b981';
    let secondaryTextColorPref = '#FFFFFF';
    let backgroundOpacity = 0.7; // Default opacity
    let currentBackgroundImage: string | null = null;
    let isCustomBackground = false;

    // Reactive variable for current session title
    $: currentSessionTitle = (() => {
        // Check if sessions is an array before using find
        if (!Array.isArray(sessions)) return 'Select Chat';
        const session = sessions.find(s => s.id === currentSessionId);
        if (!session) return 'Select Chat';

        // Use messages if available for the current session
        if (messages.length > 0) {
            const lastMessage = messages[messages.length - 1];
            const snippet = lastMessage.content.substring(0, 30); // Take first 30 chars
            // Construct title with snippet
            return `${session.title} - ${snippet}${lastMessage.content.length > 30 ? '...' : ''}`;
        }
        // Otherwise, just return the session title
        return session.title;
    })();

    function scrollToBottom(element: HTMLElement | null = chatBody) {
        setTimeout(() => {
            if (element) {
                element.scrollTop = element.scrollHeight;
            }
        }, 50); // Increased timeout slightly
    }

    // Define the closeChat function
    function closeChat() {
        isOpen = false;
        dispatch('close');
    }

    // Load all sessions
    async function loadAllSessions() {
        isLoadingSessions = true;
        errorLoadingSessions = null;
        try {
            sessions = (await get_all_writing_sessions()) ?? []; // Default to empty array if null
            // If there's an active session, ensure it's still valid
            if (currentSessionId && !sessions.some(s => s.id === currentSessionId)) {
                currentSessionId = null;
                messages = [];
            }
            if (sessions.length > 0 && currentSessionId === null) {
                console.log("->> CHAT: No session loaded, auto-loading most recent:", sessions[0].id);
                loadSessionMessages(sessions[0].id);
            }
        } catch (err) {
            console.error("Error loading sessions:", err);
            errorLoadingSessions = err instanceof Error ? err.message : "Failed to load sessions";
        } finally {
            isLoadingSessions = false;
        }
    }

    // Create new session
    async function createNewSession() {
        isLoadingSessions = true;
        errorLoadingSessions = null;
        const newTitle = `Chat ${new Date().toLocaleTimeString()}`;
        const payload: CreateSessionPayload = {
            title: newTitle,
            document_id: documentId
        };
        try {
            const newSession = await create_writing_session(payload);
            if (newSession) {
                sessions = [newSession, ...sessions];
                await loadSessionMessages(newSession.id);
            } else {
                errorLoadingSessions = "Failed to create session (null returned)";
            }
        } catch (err) {
            console.error("Error creating session:", err);
            errorLoadingSessions = err instanceof Error ? err.message : "Failed to create session";
        } finally {
            isLoadingSessions = false;
        }
    }

    // Load messages for a specific session
    async function loadSessionMessages(sessionId: number) {
        if (isLoadingMessages || currentSessionId === sessionId) return; // Prevent reloading same session or while busy

        isLoadingMessages = true;
        errorLoadingMessages = null;
        messages = []; // Clear previous messages
        currentSessionId = sessionId;

        try {
            const sessionData = await get_writing_session(sessionId);
            if (sessionData) {
                messages = sessionData.messages;
                // Scroll chat body to bottom after loading messages
                scrollToBottom(); 
            } else {
                errorLoadingMessages = "Failed to load session data (null returned)";
                currentSessionId = null; // Reset session ID if data is null
            }
        } catch (err) {
            console.error(`Error loading messages for session ${sessionId}:`, err);
            errorLoadingMessages = err instanceof Error ? err.message : "Failed to load messages";
            currentSessionId = null; // Reset session ID on error
        } finally {
            isLoadingMessages = false;
        }
    }

    // Delete a session
    async function deleteSession(sessionId: number, event: MouseEvent) {
        event.stopPropagation();

        isLoadingSessions = true; // Indicate loading while deleting
        try {
            await delete_writing_session(sessionId);
            sessions = sessions.filter(s => s.id !== sessionId);
            if (currentSessionId === sessionId) {
                currentSessionId = null;
                messages = [];
                errorLoadingMessages = null;
            }
        } catch (err) {
            console.error(`Error deleting session ${sessionId}:`, err);
            errorLoadingSessions = err instanceof Error ? err.message : "Failed to delete session";
        } finally {
             isLoadingSessions = false;
        }
    }

    // Send a message
    async function sendMessage() {
        const trimmedContent = newMessageContent.trim();
        if (!trimmedContent || !currentSessionId || isLoadingMessages) return;

        const userMessage: WritingAssistantMessage = {
            id: Date.now() + Math.random(),
            session_id: currentSessionId,
            role: 'User',
            content: trimmedContent,
            created_at: new Date().toISOString()
        };

        messages = [...messages, userMessage];
        newMessageContent = '';
        dispatch('sendMessage');
        isLoadingMessages = true;
        errorLoadingMessages = null;
        messageInput?.focus();
        scrollToBottom();

        try {
            const payload: SendMessagePayload = { content: userMessage.content };
            const assistantResponseData = await send_writing_message(currentSessionId, payload);

            if (assistantResponseData && assistantResponseData.content) {
                const assistantMessage: WritingAssistantMessage = {
                    id: Date.now(), // Temporary ID
                    session_id: currentSessionId,
                    role: 'assistant',
                    content: assistantResponseData.content,
                    created_at: new Date().toISOString()
                };
                messages = [...messages, assistantMessage];
                scrollToBottom();

                // Agentic decision for proactive diff
                const showDiffProactively = await shouldAgentShowDiffProactively(
                    assistantMessage.content,
                    { type: 'chat', userPrompt: userMessage.content },
                    "" // Pass empty string for documentContentSnippet
                );

                if (showDiffProactively) {
                    console.log('[ChatAssistant] Agent decided to proactively show diff.');
                    isApplyingSuggestion = true;
                    dispatch('showtoast', { message: 'AI suggested changes, preparing diff...', type: 'success' });
                    try {
                        const result = await apply_ai_suggestion(currentSessionId!, assistantMessage.content, documentId);
                        if (result && Array.isArray(result)) {
                            dispatch('suggestionReceived', result as SuggestedDocumentChange[]);
                        } else {
                            console.warn("[ChatAssistant] Proactive AI Apply Suggestion returned invalid data.");
                            dispatch('showtoast', { message: 'Suggestion data for diff invalid.', type: 'warning' });
                        }
                    } catch (error) {
                        console.error("[ChatAssistant] Error during proactive AI apply suggestion:", error);
                        dispatch('showtoast', { message: 'Failed to prepare proactive diff', type: 'error' });
                    } finally {
                        isApplyingSuggestion = false;
                    }
                }
            } else {
                errorLoadingMessages = "Failed to get AI response (null returned or empty content)";
            }
        } catch (err) {
            console.error("Error sending message:", err);
            errorLoadingMessages = err instanceof Error ? err.message : "Failed to get AI response";
        } finally {
            isLoadingMessages = false;
            scrollToBottom();
        }
    }

    // Function to apply the AI response
    async function applyAIResponse(suggestionContent: string) {
        if (!currentSessionId) {
            console.error("Cannot apply AI response: No active session ID.");
            dispatch('showtoast', { message: 'Cannot apply: No active session', type: 'error' });
            return;
        }
        
        console.log(`[applyAIResponse] MANUAL APPLY: Attempting to apply. Session ID: ${currentSessionId}`);
        isApplyingSuggestion = true;
        dispatch('showtoast', { message: 'Applying changes...', type: 'success' });

        try {
            const result = await apply_ai_suggestion(currentSessionId, suggestionContent, documentId);
            console.log("AI Apply Suggestion Result (Manual):", result);

            if (result && Array.isArray(result)) {
                dispatch('suggestionReceived', result as SuggestedDocumentChange[]);
            } else {
                 console.warn("AI Apply Suggestion (Manual) returned invalid data.");
                 dispatch('showtoast', { message: 'Suggestion data invalid.', type: 'warning' });
            }

        } catch (error) {
            console.error("Error applying AI suggestion (Manual):", error);
            dispatch('showtoast', { message: 'Failed to apply changes', type: 'error' });
        } finally {
            isApplyingSuggestion = false;
        }
    }

    // New loadPreferences function
    async function loadPreferences() {
        try {
            const prefs = await get_all_preferences();
            
            if (prefs) {
                preferences = prefs;
                // Set local variables for specific preferences
                preferences.forEach(pref => {
                    if (pref.preference_name === 'primary_color') {
                        primaryColorPref = pref.preference_value;
                    } else if (pref.preference_name === 'editor_background_opacity') {
                        backgroundOpacity = parseFloat(pref.preference_value); // Convert to float
                    } else if (pref.preference_name === 'secondary_color') {
                        secondaryColorPref = pref.preference_value;
                    } else if (pref.preference_name === 'primary_accent_color') {
                        primaryAccentColorPref = pref.preference_value;
                    } else if (pref.preference_name === 'secondary_accent_color') {
                        secondaryAccentColorPref = pref.preference_value;
                    } else if (pref.preference_name === 'primary_text_color') {
                        primaryTextColorPref = pref.preference_value;
                    } else if (pref.preference_name === 'secondary_text_color') {
                        secondaryTextColorPref = pref.preference_value;
                    }
                });

                // Check if background image exists
                const { imageUrl, isCustom } = await check_background_image();
                if (imageUrl) {
                    currentBackgroundImage = imageUrl;
                }
                isCustomBackground = isCustom;
            } else {
                console.error('Failed to load preferences');
                dispatch('showtoast', { message: 'Failed to load preferences', type: 'error' });
            }
        } catch (error) {
            console.error('Error loading preferences:', error);
            dispatch('showtoast', { message: 'An error occurred while loading preferences', type: 'error' });
        }
    }

    // Mount logic
    onMount(async () => {
        if (isOpen) {
            await loadAllSessions();
            await loadPreferences(); // Call loadPreferences here
        }
        // Optional: Add focus to input when chat opens?
        if (isOpen && messageInput) messageInput.focus();
    });

    // Function to copy text to clipboard
    async function copyToClipboard(text: string, buttonElement: HTMLButtonElement) {
        try {
            await navigator.clipboard.writeText(text);
            // Optional: Provide feedback (e.g., change icon briefly)
            const originalIcon = buttonElement.innerHTML;
            buttonElement.innerHTML = '<i class="bi bi-check-lg text-success"></i>'; // Checkmark icon
            setTimeout(() => {
                buttonElement.innerHTML = originalIcon; // Restore original icon
            }, 1500); // Restore after 1.5 seconds
        } catch (err) {
            console.error('Failed to copy text: ', err);
            // Optional: Show error feedback to user
        }
    }

    export function sendProgrammaticMessage(messageContent: string, role: 'user' | 'assistant') {
        if (!currentSessionId) {
            console.warn("Cannot send programmatic message: No active session.");
            // Optionally dispatch a toast error back to the parent?
            // dispatch('showtoast', { message: 'Chat session not active.', type: 'warning' });
            return;
        }

        const newMessage: WritingAssistantMessage = {
            id: Date.now() + Math.random(), // Temporary unique ID for UI
            session_id: currentSessionId,
            role: role === 'user' ? 'User' : 'assistant', // Map role correctly
            content: messageContent,
            created_at: new Date().toISOString()
        };

        messages = [...messages, newMessage];
        scrollToBottom(); // Scroll down to show the new message
        console.log(`Programmatic message added (role: ${role}):`, messageContent.substring(0, 50) + '...');
    }

</script>
{#if isOpen}
<!-- Use 'showing' class for transitions if desired -->
<div class="offcanvas offcanvas-end show text-bg-dark" tabindex="-1" id="chatAssistantOffcanvas" aria-labelledby="chatAssistantLabel">
    <div class="offcanvas-header border-bottom" style="border-color: var(--secondary-color);">
        <!-- Replace static title with Dropdown for sessions -->
        <div class="dropdown flex-grow-1 me-2">
            <button class="btn btn-dark dropdown-toggle w-100 text-start" type="button" id="sessionDropdownMenuButton" data-bs-toggle="dropdown" aria-expanded="false">
                {currentSessionTitle}
            </button>
            <ul class="dropdown-menu dropdown-menu-dark w-100" aria-labelledby="sessionDropdownMenuButton">
                {#if isLoadingSessions}
                    <li><span class="dropdown-item-text">Loading...</span></li>
                {:else if errorLoadingSessions}d
                     <li><span class="dropdown-item-text text-danger">Error loading</span></li>
                {:else}
                    {#each sessions as session (session.id)}
                        <li class="d-flex align-items-center" class:active-row={currentSessionId === session.id}>
                            <button 
                                class="dropdown-item flex-grow-1 {currentSessionId === session.id ? 'active' : ''}" 
                                on:click={() => loadSessionMessages(session.id)}
                            >
                                {session.title}
                                {#if session.last_message_snippet}
                                    <span class="text-muted fst-italic snippet-text"> - {session.last_message_snippet}</span>
                                {/if}
                            </button>
                            <button 
                                class="btn btn-sm btn-danger delete-session-btn" 
                                on:click|stopPropagation={(e) => deleteSession(session.id, e)}
                                title="Delete Session"
                                aria-label="Delete session {session.title}"
                            >
                                &times;
                            </button>
                        </li>
                    {/each}
                    {#if sessions.length > 0}
                        <li><hr class="dropdown-divider"></li>
                    {/if}
                    <li><button class="dropdown-item" on:click={createNewSession}><i class="bi bi-plus-lg me-1"></i> New Chat</button></li>
                {/if}
            </ul>
        </div>
        <!-- <h5 class="offcanvas-title" id="chatAssistantLabel">AI Writing Assistant</h5> -->
        <button type="button" class="btn-close btn-close-white" aria-label="Close" on:click={closeChat}></button>
    </div>
    <div class="offcanvas-body d-flex flex-column p-0">
        <!-- Session List Removed -->

         <!-- Message Display Area -->
        <div bind:this={chatBody} class="chat-body flex-grow-1 p-3" style="overflow-y: auto;">
            {#if isLoadingMessages && messages.length === 0}
                <div class="text-center text-muted">Loading messages...</div>
            {:else if errorLoadingMessages}
                <div class="alert alert-danger p-2">{errorLoadingMessages}</div>
            {:else if !currentSessionId}
                <!-- Removed placeholder text -->
            {:else if messages.length === 0 && !isLoadingMessages}
                 <div class="text-center text-muted fst-italic">No messages in this session yet.</div>
            {:else}
                {#each messages as message (message.id)}
                    <!-- Apply conditional class based on role (case-sensitive check) -->
                    <div class="message mb-2 {message.role === 'User' ? 'message-user' : 'message-assistant'}">
                        <div class="message-content p-2 rounded position-relative">
                            <!-- Basic Markdown-like rendering for newlines -->
                            {#each message.content.split('\n') as line, i}
                                {line}{#if i < message.content.split('\n').length - 1}<br/>{/if}
                            {/each}

                            <!-- Copy Button for Assistant messages -->
                            {#if message.role !== 'User'}
                                <button 
                                    class="btn btn-sm copy-button" 
                                    title="Copy to clipboard"
                                    style="color: var(--primary-color)"
                                    aria-label="Copy assistant message to clipboard"
                                    on:click={(e) => copyToClipboard(message.content, e.currentTarget)}
                                >
                                    <i class="bi bi-clipboard"></i>
                                </button>
                                <!-- Apply Button for Assistant messages -->
                                <button 
                                    class="btn btn-sm apply-ai-button" 
                                    title="Apply suggestion"
                                    style="color: var(--primary-color)"
                                    aria-label="Apply AI suggestion to document"
                                    on:click={() => applyAIResponse(message.content)}
                                    disabled={isApplyingSuggestion}
                                >
                                    {#if isApplyingSuggestion} 
                                        <i class="bi bi-arrow-repeat spin"></i>
                                    {:else}
                                        <i class="bi bi-check-circle"></i>
                                    {/if}
                                </button>
                            {/if}
                        </div>

                    </div>
                {/each}
                {#if isLoadingMessages && messages.length > 0}
                    <div class="message message-assistant">
                         <div class="message-content p-2 rounded">
                             <span class="spinner-border spinner-border-sm" role="status" aria-hidden="true"></span> Thinking...
                         </div>
                    </div>
                {/if}
            {/if}
        </div>

        <!-- Message Input Area -->
        {#if currentSessionId}
            <div class="input-group p-3">
                <input
                    bind:this={messageInput}
                    type="text"
                    class="form-control bg-transparent text-white "
                    
                    placeholder="Ask the AI..."
                    bind:value={newMessageContent}
                    on:keydown={(e) => { if (e.key === 'Enter') sendMessage(); }}
                    disabled={isLoadingMessages}
                />
                 <button class="btn" style="background-color: var(--primary-color)" on:click={sendMessage} disabled={isLoadingMessages || !newMessageContent.trim()}>
                    {#if isLoadingMessages}
                        <span class="spinner-border spinner-border-sm" role="status" aria-hidden="true"></span>
                    {:else}
                        <i class="bi bi-send" style="color: var(--secondary-color)"></i>
                    {/if}
                </button>
            </div>
        {/if}
    </div>
</div>
{/if}

<style>
    :root {
        --primary-color: #0A1721;
        --secondary-color: #10b981;
        --primary-accent-color: #10b981;
        --secondary-accent-color: #808080;
        --primary-text-color: #10b981;
        --secondary-text-color: #FFFFFF;
        --primary-color-rgba: rgba(10, 23, 33, 0.5);
        --editor-background-opacity: 0.7;
    }

    .offcanvas-end {
        width: 450px
    }
    .offcanvas.offcanvas-end.show {
        background-color: var(--primary-color) !important;
        backdrop-filter: blur(8px);
        -webkit-backdrop-filter: blur(8px);
        border-radius: 8px;
        border-color: var(--secondary-color);
        top: 165px;
        height: calc(100vh - 220px);
        max-height: calc(100vh - 220px);
        min-height: 300px;
        transition: transform 0.4s ease-out;
        margin-right: 50px;
        z-index: 0;
        display: flex;
        flex-direction: column;
    }
    .offcanvas-body.d-flex.flex-column.p-0 {
        flex: 1 1 auto;
        display: flex;
        flex-direction: column;
        min-height: 0;
        height: 100%;
        padding: 0;
    }
    .chat-body {
        background-color: var(--primary-color);
        flex: 1 1 auto;
        overflow-y: auto;
        padding-bottom: 110px; /* Increased to ensure input is always visible above footer/status bar */
        min-height: 0;
    }
    /* Make the input group sticky at the bottom of the chat */
    .input-group.p-3 {
        position: sticky;
        bottom: 0;
        z-index: 2;
        border-radius: 0 0 8px 8px;
        border-color: var(--secondary-color);
    }
    .offcanvas-header,
    /* Remove session list container styles */
    .input-group { /* Apply border to input area */
        border-color: var(--secondary-color);
    }

    .message-assistant .message-content {
        background-color: var(--secondary-color); /* Darker gray for assistant - made slightly transparent */
        color: var(--secondary-text-color);
        border-radius: 15px 15px 15px 5px; /* Chat bubble style */
        margin-left:1px
    }

    /* Reduce font size for message content */
    .message-content {
        font-size: 0.9rem; /* Adjust as needed */
        line-height: 1.4; /* Adjust line height for readability */
    }
    
    /* Adjust active dropdown item style if needed */
    .dropdown-item.active {
        background-color: #595f68 !important; /* Ensure active color overrides */
        border-color: #0a58ca !important;
    }

    /* Style the dropdown toggle button */
    .offcanvas-header .dropdown-toggle {
        background-color: rgba(255, 255, 255, 0.1); /* Subtle background */
        border: none;
        font-weight: 500;
    }
    .offcanvas-header .dropdown-toggle:hover,
    .offcanvas-header .dropdown-toggle:focus {
        background-color: var(--secondary-color)
    }

    ::-webkit-scrollbar {
        width: 8px;
    }
    ::-webkit-scrollbar-track {
        background: #212529;
    }
    ::-webkit-scrollbar-thumb {
        background: #555;
        border-radius: 4px;
    }
    ::-webkit-scrollbar-thumb:hover {
        background: #777;
    }

    .message {
        display: flex;
        flex-direction: column;
        max-width: 80%; /* Limit message width */
        word-wrap: break-word; /* Ensure long words break */
    }

    .message-user {
        align-items: flex-end; /* Align user messages to the right */
        margin-left: auto; /* Push to the right */
    }

    .message-assistant {
        align-items: flex-start; /* Align assistant messages to the left */
        margin-right: auto; /* Push to the left */
    }

    .message-user .message-content {
        background-color: var(--primary-accent-color); /* Bootstrap success green */
        color: var(--secondary-text-color);
        border-radius: 15px 15px 5px 15px; /* Chat bubble style */
    }

    /* Ensure flex properties on the li */
    .dropdown-menu li.d-flex {
        padding: 0; /* Remove padding from li if button handles it */
    }

    /* Style the main session button within the li */
    .dropdown-menu li .dropdown-item.flex-grow-1 {
        text-align: left; /* Ensure text aligns left */
        border-radius: 0; /* Remove individual button radius if needed */
         /* Inherit padding or set explicitly */
        padding: 0.5rem 1rem; 
        /* Add overflow handling */
        overflow: hidden;
        white-space: nowrap;
        text-overflow: ellipsis;
        /* Ensure button takes available space but respects delete btn */
        min-width: 0; /* Allow button to shrink */
    }

    /* Style the snippet text */
    .snippet-text {
        color: rgba(255, 255, 255, 0.7) !important; /* Lighter color, override text-muted */
        margin-left: 0.25rem; /* Add small space before snippet */
    }

    /* Style the delete button */
    .delete-session-btn {
        background-color: transparent;
        border: none;
        color: #adb5bd; /* Lighter gray for icon */
        padding: 0.5rem 0.8rem; /* Adjust padding */
        margin-left: 5px; /* Space between title and button */
        line-height: 1; /* Ensure '×' aligns well */
        font-size: 1.2rem; /* Make '×' slightly larger */
        opacity: 0.6;
        transition: opacity 0.2s ease, color 0.2s ease;
    }

    .delete-session-btn:hover {
        color: #dc3545; /* Bootstrap danger red on hover */
        opacity: 1;
    }

    /* Hover effect for the li - not when active */
    .dropdown-menu li:not(.active-row):hover {
        background-color: rgba(255, 255, 255, 0.08); /* Apply hover directly to li */
    }
    /* Ensure button background/color is correct on hover */
    .dropdown-menu li:not(.active-row):hover > .dropdown-item {
        background-color: transparent;
        color: var(--bs-dropdown-link-hover-color, white);
    }
    
    /* Style the active row (li) */
    .dropdown-menu li.active-row {
        background-color: var(--primary-color) !important; 
    }

    /* Style the active button within the active row */
    .dropdown-menu li.active-row .dropdown-item.active {
        background-color: transparent !important; /* Make button background transparent */
        color: white !important; /* Keep text white */
    }

    /* Style the delete button within the active row for visibility */
    .dropdown-menu li.active-row .delete-session-btn {
        color: var(--secondary-accent-color);
        opacity: 0.7;
    }
    .dropdown-menu li.active-row .delete-session-btn:hover {
        color: var(--bs-danger);
        opacity: 1;
    }

    /* Style for the copy button */
    .copy-button {
        position: absolute;
        top: 2px; /* Adjust as needed */
        right: 2px; /* Adjust as needed */
        padding: 0.1rem 0.3rem; /* Smaller padding */
        font-size: 0.8rem; /* Smaller icon */
        background-color: transparent; 
        border: none;
        color: rgba(255, 255, 255, 0.4); /* Dim color */
        opacity: 0.4; /* Initially less visible */
        transition: opacity 0.2s ease, color 0.2s ease;
    }
    .message-assistant .message-content:hover .copy-button {
        opacity: 1; /* Fully visible on hover */
        color: rgba(255, 255, 255, 0.8); /* Brighter on hover */
    }
    .copy-button:hover {
        color: white !important; /* White on direct button hover */
         background-color: rgba(0, 0, 0, 0.2); /* Slight background on hover */
    }

    /* Style for the Apply AI button */
    .apply-ai-button {
        position: absolute;
        top: 2px;
        right: 30px; /* Position next to copy button */
        padding: 0.1rem 0.3rem;
        font-size: 0.8rem;
        background-color: transparent;
        border: none;
        color: rgba(255, 255, 255, 0.4);
        opacity: 0.4;
        transition: opacity 0.2s ease, color 0.2s ease;
    }
    .message-assistant .message-content:hover .apply-ai-button {
        opacity: 1;
        color: rgba(144, 238, 144, 0.8); /* Light green */
    }
    .apply-ai-button:hover {
        color: #90ee90 !important; /* Brighter light green on hover */
        background-color: rgba(0, 0, 0, 0.2); 
    }

</style>
