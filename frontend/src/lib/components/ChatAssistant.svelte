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
    let sessionsList: HTMLUListElement;

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
            // Scroll sessions list to top after loading
            setTimeout(() => sessionsList?.scrollTo({ top: 0 }), 50);
        }
    }

    // Create new session
    async function createNewSession() {
        isLoadingSessions = true;
        errorLoadingSessions = null;
        const newTitle = `Chat ${documentId ? ` (Doc ${documentId}) ` : ''} ${new Date().toLocaleTimeString()}`;
        const payload: CreateSessionPayload = {
            title: newTitle,
            document_id: documentId
        };
        try {
            const newSession = await create_writing_session(payload);
            if (newSession) {
                sessions = [newSession, ...sessions];
                await loadSessionMessages(newSession.id);
                // Scroll sessions list to top to show the new one
                setTimeout(() => sessionsList?.scrollTo({ top: 0, behavior: 'smooth' }), 50);
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

        // Optional: Add a confirmation dialog
        if (!confirm("Are you sure you want to delete this chat session?")) {
             return;
        }

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
            role: 'user',
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
        <h5 class="offcanvas-title" id="chatAssistantLabel">AI Writing Assistant</h5>
        <button type="button" class="btn-close btn-close-white" aria-label="Close" on:click={closeChat}></button>
    </div>
    <div class="offcanvas-body d-flex flex-column p-0">
        <!-- Session List -->
        <div class="session-list-container p-3 border-bottom border-secondary">
            <div class="d-flex justify-content-between align-items-center mb-2">
                <h6 class="mb-0 text-success">Chat Sessions</h6>
                 <button class="btn btn-sm btn-success" on:click={createNewSession} title="New Chat">
                    <i class="bi bi-plus-lg me-1"></i> New
                 </button>
            </div>
             <div class="session-list" style="height: 180px; overflow-y: auto;">
                 {#if isLoadingSessions}
                     <div class="text-center text-muted pt-3">Loading sessions...</div>
                 {:else if errorLoadingSessions}
                      <div class="alert alert-danger p-2">{errorLoadingSessions}</div>
                 {:else if sessions.length === 0}
                     <div class="text-center text-muted fst-italic pt-3">No sessions yet.</div>
                 {:else}
                     <ul bind:this={sessionsList} class="list-group list-group-flush">
                         {#each sessions as session (session.id)}
                             <div
                                 class="list-group-item list-group-item-action bg-dark text-white border-secondary d-flex justify-content-between align-items-center px-2 py-1 {currentSessionId === session.id ? 'active' : ''}"
                                 style="cursor: pointer;"
                                 on:click={() => loadSessionMessages(session.id)}
                                 on:keydown={(e) => { if (e.key === 'Enter' || e.key === ' ') loadSessionMessages(session.id); }}
                                 role="button"
                                 tabindex="0"
                                 title={session.title}
                             >
                                  <span class="text-truncate me-2">{session.title}</span>
                                  <button type="button"
                                     class="btn btn-sm text-danger border-0 p-0"
                                     on:click={(e) => deleteSession(session.id, e)}
                                     title="Delete Session"
                                     aria-label="Delete session {session.title}"
                                  >
                                     <i class="bi bi-x-lg"></i>
                                 </button>
                             </div>
                         {/each}
                     </ul>
                 {/if}
             </div>
        </div>

         <!-- Message Display Area -->
        <div bind:this={chatBody} class="chat-body flex-grow-1 p-3" style="overflow-y: auto;">
            {#if isLoadingMessages && messages.length === 0}
                 <div class="text-center text-muted pt-5"><span class="spinner-border spinner-border-sm text-success me-2"></span>Loading messages...</div>
            {:else if errorLoadingMessages}
                 <div class="alert alert-danger p-2">{errorLoadingMessages}</div>
            {:else if !currentSessionId}
                 <div class="text-center text-muted fst-italic pt-5">Select or start a session.</div>
            {:else if messages.length === 0}
                 <div class="text-center text-muted fst-italic pt-5">Send a message to start the chat.</div>
            {:else}
                 {#each messages as message (message.id)}
                     <div class="chat-message mb-3 d-flex {message.role === 'user' ? 'justify-content-end' : 'justify-content-start'}">
                         <div
                              class="p-2 px-3 rounded-3 shadow-sm"
                              style="max-width: 85%; word-wrap: break-word; {message.role === 'user' ? 'background-color: var(--bs-success); color: var(--bs-dark);' : 'background-color: var(--bs-secondary);'}"
                          >
                              {@html message.content.replace(/\\n/g, '<br/>')} <!-- Render newlines -->
                          </div>
                     </div>
                 {/each}
                  {#if isLoadingMessages && messages.length > 0}
                      <div class="chat-message mb-3 d-flex justify-content-start">
                          <div class="p-2 px-3 rounded-3 shadow-sm bg-secondary" style="max-width: 85%;">
                             <span class="spinner-grow spinner-grow-sm" role="status" aria-hidden="true"></span>
                             <span class="ms-1 fst-italic">Assistant is typing...</span>
                         </div>
                     </div>
                 {/if}
            {/if}
        </div>

        <!-- Input Area -->
        <div class="chat-input p-3 border-top border-secondary">
            <form on:submit|preventDefault={sendMessage}>
                <div class="input-group">
                    <input
                        bind:this={messageInput}
                        type="text"
                        class="form-control bg-black text-white border-secondary shadow-none"
                        placeholder={currentSessionId ? "Ask the AI..." : "Select a session"}
                        aria-label="Chat input"
                        bind:value={newMessageContent}
                        disabled={!currentSessionId || isLoadingMessages}
                        autocomplete="off"
                    />
                    <button
                        class="btn btn-success"
                        type="submit"
                        disabled={!currentSessionId || isLoadingMessages || !newMessageContent.trim()}
                    >
                        {#if isLoadingMessages && messages.length > 0 && messages[messages.length -1].role === 'user'}
                             <span class="spinner-border spinner-border-sm" role="status" aria-hidden="true"></span>
                        {:else}
                             <i class="bi bi-send"></i> <!-- Changed icon -->
                        {/if}
                    </button>
                </div>
            </form>
        </div>
    </div>
</div>
{/if}


<style>
    .offcanvas-end {
        width: 350px
    }
    .offcanvas {
        background-color: rgba(10, 23, 33, 0.95);
        backdrop-filter: blur(5px);
        -webkit-backdrop-filter: blur(5px);
        border-radius: 8px 0 0 8px;
        top: 70px;
        height: calc(100vh - 70px - 50px);
        transition: transform 0.4s ease-out;
    }
    .offcanvas-header,
    .session-list-container,
    .chat-input {
        border-color: rgba(16, 185, 129, 0.3);
    }
    .session-list-container {
        background-color: transparent;
    }
     .session-list .list-group-item {
        transition: background-color 0.2s ease;
     }
     .session-list .list-group-item:hover {
        background-color: #343a40;
     }
     .session-list .list-group-item.active {
        background-color: var(--bs-success);
        border-color: var(--bs-success);
        color: var(--bs-dark);
    }
     .session-list .list-group-item.active .btn {
        color: var(--bs-danger);
     }
     .session-list .list-group-item .btn:hover {
        color: var(--bs-danger);
        opacity: 0.8;
     }

    .chat-body {
        background-color: #111;
    }
    .chat-input input:focus {
        border-color: var(--bs-success);
        box-shadow: 0 0 0 0.25rem rgba(var(--bs-success-rgb), 0.25);
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
</style>
