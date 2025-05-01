<script lang="ts">
    import { onMount, createEventDispatcher } from 'svelte';
    import { get_all_writing_sessions, create_writing_session, get_writing_session, send_writing_message, delete_writing_session } from '$lib/ts/ai';
    import type { WritingAssistantSession, WritingAssistantMessage, SessionWithMessages, CreateSessionPayload, SendMessagePayload } from '$lib/ts/ai'; // Import all needed types

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

    // Reactive variable for current session title
    $: currentSessionTitle = sessions.find(s => s.id === currentSessionId)?.title || 'Select Chat';

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
            sessions = await get_all_writing_sessions();
            // If there's an active session, ensure it's still valid
            if (currentSessionId && !sessions.some(s => s.id === currentSessionId)) {
                currentSessionId = null;
                messages = [];
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
            id: Date.now() + Math.random(), // Temporary unique ID for UI
            session_id: currentSessionId,
            role: 'User',
            content: trimmedContent,
            created_at: new Date().toISOString()
        };

        messages = [...messages, userMessage];
        newMessageContent = ''; // Clear input immediately
        isLoadingMessages = true;
        errorLoadingMessages = null;

        // Focus input again after sending
        messageInput?.focus(); 
        scrollToBottom(); // Scroll down to show user message

        try {
            const payload: SendMessagePayload = { content: userMessage.content };
            const assistantResponse = await send_writing_message(currentSessionId, payload);
            if (assistantResponse) {
                // Construct the full message object using only role and content from response
                const fullResponse: WritingAssistantMessage = {
                    id: Date.now(), // Generate a temporary ID for the UI
                    session_id: currentSessionId,
                    role: 'assistant', // We know the response is from the assistant
                    content: assistantResponse.content || '', // Use content from response
                    created_at: new Date().toISOString() // Use current time
                };
                messages = [...messages, fullResponse];
            } else {
                errorLoadingMessages = "Failed to get AI response (null returned)";
            }
        } catch (err) {
            console.error("Error sending message:", err);
            errorLoadingMessages = err instanceof Error ? err.message : "Failed to get AI response";
        } finally {
            isLoadingMessages = false;
            scrollToBottom(); // Scroll down to show assistant message or error
        }
    }

    // Mount logic
    onMount(() => {
        if (isOpen) {
            loadAllSessions();
        }
        // Optional: Add focus to input when chat opens?
        // $: if (isOpen && messageInput) messageInput.focus();
    });

</script>
{#if isOpen}
<!-- Use 'showing' class for transitions if desired -->
<div class="offcanvas offcanvas-end show text-bg-dark" tabindex="-1" id="chatAssistantOffcanvas" aria-labelledby="chatAssistantLabel">
    <div class="offcanvas-header border-bottom border-secondary">
        <!-- Replace static title with Dropdown for sessions -->
        <div class="dropdown flex-grow-1 me-2">
            <button class="btn btn-dark dropdown-toggle w-100 text-start" type="button" id="sessionDropdownMenuButton" data-bs-toggle="dropdown" aria-expanded="false">
                {currentSessionTitle}
            </button>
            <ul class="dropdown-menu dropdown-menu-dark w-100" aria-labelledby="sessionDropdownMenuButton">
                {#if isLoadingSessions}
                    <li><span class="dropdown-item-text">Loading...</span></li>
                {:else if errorLoadingSessions}
                     <li><span class="dropdown-item-text text-danger">Error loading</span></li>
                {:else}
                    {#each sessions as session (session.id)}
                        <li class="d-flex align-items-center" class:active-row={currentSessionId === session.id}>
                            <button 
                                class="dropdown-item flex-grow-1 {currentSessionId === session.id ? 'active' : ''}" 
                                on:click={() => loadSessionMessages(session.id)}
                            >
                                {session.title}
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
                        <div class="message-content p-2 rounded">
                            <!-- Basic Markdown-like rendering for newlines -->
                            {#each message.content.split('\n') as line, i}
                                {line}{#if i < message.content.split('\n').length - 1}<br/>{/if}
                            {/each}
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
            <div class="input-group p-3 border-top border-secondary">
                <input
                    bind:this={messageInput}
                    type="text"
                    class="form-control bg-transparent text-white border-secondary"
                    placeholder="Ask the AI..."
                    bind:value={newMessageContent}
                    on:keydown={(e) => { if (e.key === 'Enter') sendMessage(); }}
                    disabled={isLoadingMessages}
                />
                 <button class="btn btn-success" on:click={sendMessage} disabled={isLoadingMessages || !newMessageContent.trim()}>
                    {#if isLoadingMessages}
                        <span class="spinner-border spinner-border-sm" role="status" aria-hidden="true"></span>
                    {:else}
                        <i class="bi bi-send"></i>
                    {/if}
                </button>
            </div>
        {/if}
    </div>
</div>
{/if}


<style>
    .offcanvas-end {
        width: 450px
    }
    .offcanvas.offcanvas-end.show { /* Increased specificity */
        /* Make background semi-transparent and add blur */
        background-color: rgba(10, 23, 33, 0.60) !important; /* Added !important */
        backdrop-filter: blur(8px); /* Increased blur */
        -webkit-backdrop-filter: blur(8px);
        border-radius: 8px; /* Round all corners */
        top: 155px; /* Maintain position below navbar */
        height: 770px;
        transition: transform 0.4s ease-out;
        margin-right: 50px; /* Add right margin */
    }
    .offcanvas-header,
    /* Remove session list container styles */
    .input-group { /* Apply border to input area */
        border-color: rgba(22, 163, 74, 0.4); /* Adjusted green border */
    }

    .chat-body {
        background-color: transparent; /* Make chat body background transparent */
    }
    .message-assistant .message-content {
        background-color: rgba(73, 80, 87, 0.9); /* Darker gray for assistant - made slightly transparent */
        color: white;
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
        background-color: #0a58ca !important; /* Ensure active color overrides */
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
        background-color: rgba(255, 255, 255, 0.15);
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
        background-color: #198754; /* Bootstrap success green */
        color: white;
        border-radius: 15px 15px 5px 15px; /* Chat bubble style */
    }

    /* --- START: Enhanced Dropdown Styles --- */

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
        background-color: var(--bs-success) !important; 
    }

    /* Style the active button within the active row */
    .dropdown-menu li.active-row .dropdown-item.active {
        background-color: transparent !important; /* Make button background transparent */
        color: white !important; /* Keep text white */
    }

    /* Style the delete button within the active row for visibility */
    .dropdown-menu li.active-row .delete-session-btn {
        color: rgba(255, 255, 255, 0.7);
        opacity: 0.7;
    }
    .dropdown-menu li.active-row .delete-session-btn:hover {
        color: var(--bs-danger);
        opacity: 1;
    }
</style>
