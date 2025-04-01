<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { get_document, update_document, setup_auto_save, get_project_from_document } from '$lib/ts/document';
  import { get_project_documents } from '$lib/ts/project';
  import { goto } from '$app/navigation';
  import logo from '$lib/assets/logo.png';
  import backgroundImage from '$lib/assets/editor-background.jpg';
  
  // Document state
  let documentId = $page.params.id;
  let documentData: any = null;
  let loading = true;
  let error = false;
  
  // Project state
  let projectDocuments: any[] = [];
  let currentDocumentIndex = -1;
  let projectDocumentsMap = new Map(); // Map to store preloaded documents
  
  // Editor state
  let editorContent = '';
  let editorMode: 'NORMAL' | 'INSERT' | 'COMMAND' = 'NORMAL';
  let cursorLine = 1;
  let cursorColumn = 1;
  let editorElement: HTMLTextAreaElement;
  
  // Add this for line numbers
  let lines: string[] = [''];
  let activeLineIndex = 0;
  
  // Add these variables for animation
  let isAnimating = false;
  let slideDirection = ''; // 'left' or 'right'
  let previousDocumentContent = '';
  let previousDocumentLines: string[] = [];
  let previousActiveLineIndex = 0;
  let animationHeight = 0; // Store the height for consistent animation
  
  // Cursor prompts
  let showCursorPrompt = true;
  let cursorPrompts = {
    'NORMAL': [
      { key: 'i', description: 'Enter insert mode' },
      { key: 'Esc', description: 'Return to normal mode' },
      { key: ':', description: 'Enter command mode' },
      { key: 'h/j/k/l', description: 'Move cursor left/down/up/right' },
      { key: 'w', description: 'Move forward by word' },
      { key: 'b', description: 'Move backward by word' },
      { key: 'x', description: 'Delete character' },
      { key: 'dd', description: 'Delete line' },
      { key: 'yy', description: 'Copy line' },
      { key: 'p', description: 'Paste after cursor' }
    ],
    'INSERT': [
      { key: 'Esc', description: 'Return to normal mode' },
      { key: 'Type', description: 'Edit text' },
      { key: 'Enter', description: 'New line' },
      { key: 'Backspace', description: 'Delete previous character' }
    ],
    'COMMAND': [
      { key: ':w', description: 'Save document' },
      { key: ':q', description: 'Quit editor' },
      { key: ':wq', description: 'Save and quit' },
      { key: ':help', description: 'Show help' },
      { key: 'Esc', description: 'Return to normal mode' }
    ]
  } as const;
  
  // Function to switch to another document with animation
  async function switchDocument(docId: number) {
    try {
      console.log(`Switching to document ${docId} from ${documentId}`);
      
      // Don't switch if already animating
      if (isAnimating) return;
      
      // Save current document before switching
      if (documentData) {
        console.log('Saving current document before switching');
        documentData.content = editorContent;
        await update_document(documentData);
      }
      
      // Check if we already have the document loaded
      if (projectDocumentsMap.has(docId)) {
        console.log('Using preloaded document data');
        
        // Determine slide direction based on document indices
        const currentIndex = projectDocuments.findIndex(doc => doc.id.toString() === documentId);
        const newIndex = projectDocuments.findIndex(doc => doc.id === docId);
        
        if (currentIndex < newIndex) {
          // Moving to a higher number - slide left
          slideDirection = 'left';
        } else {
          // Moving to a lower number - slide right
          slideDirection = 'right';
        }
        
        // Store current document content for animation
        previousDocumentContent = editorContent;
        previousDocumentLines = [...lines];
        previousActiveLineIndex = activeLineIndex;
        
        // Store current editor height for smooth animation
        if (editorElement && editorElement.parentElement) {
          animationHeight = Math.max(
            editorElement.parentElement.offsetHeight,
            editorElement.scrollHeight
          );
        }
        
        // Start animation
        isAnimating = true;
        
        // Update document ID in URL without full page reload
        window.history.pushState({}, '', `/document/${docId}`);
        documentId = docId.toString();
        
        // Load the preloaded document data
        documentData = projectDocumentsMap.get(docId);
        editorContent = documentData.content || '';
        lines = editorContent.split('\n');
        
        // Update current document index
        currentDocumentIndex = projectDocuments.findIndex(doc => doc.id === docId);
        
        // Wait for animation to complete
        setTimeout(() => {
          isAnimating = false;
          slideDirection = '';
          previousDocumentContent = '';
          previousDocumentLines = [];
          animationHeight = 0;
          // Adjust textarea height
          setTimeout(adjustTextareaHeight, 0);
        }, 300); // Match this with CSS transition duration
        
        return;
      }
      
      // If document not preloaded, navigate to it the traditional way
      console.log(`Document not preloaded, navigating to /document/${docId}`);
      window.location.href = `/document/${docId}`;
    } catch (error) {
      console.error('Error switching document:', error);
      isAnimating = false;
    }
  }
  
  // Load project documents and preload their content
  async function loadProjectDocuments() {
    console.log('Loading project documents, document data:', documentData);
    
    if (!documentData || !documentData.project_id) {
      console.log('No project_id found in document data');
      return;
    }
    
    try {
      console.log(`Fetching documents for project ${documentData.project_id}`);
      const docs = await get_project_documents(documentData.project_id);
      console.log('Received project documents:', docs);
      
      if (docs && docs.length > 0) {
        projectDocuments = docs;
        
        // Find current document index
        currentDocumentIndex = projectDocuments.findIndex(doc => doc.id.toString() === documentId);
        console.log(`Current document index: ${currentDocumentIndex}`);
        
        // Preload all documents in the project
        await preloadDocuments();
      } else {
        console.log('No documents found in project');
        projectDocuments = []; // Ensure it's an empty array
      }
    } catch (e) {
      console.error('Error loading project documents:', e);
      projectDocuments = []; // Ensure it's an empty array on error
    }
  }
  
  // Preload all documents in the project
  async function preloadDocuments() {
    console.log('Preloading project documents');
    
    // Add current document to the map
    projectDocumentsMap.set(parseInt(documentId), documentData);
    
    // Preload other documents
    for (const doc of projectDocuments) {
      if (doc.id.toString() !== documentId) {
        try {
          console.log(`Preloading document ${doc.id}`);
          const docData = await get_document(doc.id);
          if (docData) {
            projectDocumentsMap.set(doc.id, docData);
          }
        } catch (e) {
          console.error(`Error preloading document ${doc.id}:`, e);
        }
      }
    }
    
    console.log('Preloading complete, documents in cache:', projectDocumentsMap.size);
  }
  
  // Handle key events to switch modes
  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      editorMode = 'NORMAL';
    } else if (editorMode === 'NORMAL') {
      if (event.key === 'i') {
        editorMode = 'INSERT';
        event.preventDefault();
      } else if (event.key === ':') {
        editorMode = 'COMMAND';
        event.preventDefault();
      }
    }
    
    // Update cursor position
    if (editorElement) {
      const position = editorElement.selectionStart;
      const text = editorElement.value;
      const lines = text.substr(0, position).split('\n');
      cursorLine = lines.length;
      cursorColumn = lines[lines.length - 1].length + 1;
    }
    
    // Update active line
    if (editorElement) {
      const position = editorElement.selectionStart;
      const text = editorElement.value;
      const textBeforeCursor = text.substring(0, position);
      activeLineIndex = (textBeforeCursor.match(/\n/g) || []).length;
    }
  }
  
  // Update the adjustTextareaHeight function
  function adjustTextareaHeight() {
    if (editorElement) {
      // Reset height to auto to get the correct scrollHeight
      editorElement.style.height = 'auto';
      
      // Set the height to match the content
      const newHeight = Math.max(editorElement.scrollHeight, editorElement.parentElement?.clientHeight || 0);
      editorElement.style.height = newHeight + 'px';
    }
  }
  
  // Load document data
  onMount(async () => {
    try {
      documentData = await get_document(parseInt(documentId));
      if (documentData) {
        editorContent = documentData.content || '';
        lines = editorContent.split('\n');
        
        // Check if document is part of a project
        if (!documentData.project_id) {
          // If project_id is not in document data, try to get it from the API
          const projectData = await get_project_from_document(parseInt(documentId));
          if (projectData && projectData.project_id) {
            // Add project_id to document data
            documentData.project_id = projectData.project_id;
          }
        }
        
        // Load project documents if this document is part of a project
        await loadProjectDocuments();
        
        // Set up auto-save
        const cleanup = setup_auto_save(documentData, (success) => {
          // Handle save status if needed
        });
        
        // Initial height adjustment
        setTimeout(adjustTextareaHeight, 0);
      } else {
        error = true;
      }
    } catch (e) {
      console.error('Error loading document:', e);
      error = true;
    } finally {
      loading = false;
    }
  });
  
  // Update handleInput to also adjust height
  function handleInput() {
    if (documentData) {
      documentData.content = editorContent;
      
      // Update lines array for line numbers
      lines = editorContent.split('\n');
      
      // Update active line
      if (editorElement) {
        const position = editorElement.selectionStart;
        const textBeforeCursor = editorContent.substring(0, position);
        activeLineIndex = (textBeforeCursor.match(/\n/g) || []).length;
      }
      
      // Adjust textarea height
      adjustTextareaHeight();
    }
  }

  // Handle cleanup in onDestroy instead
  onDestroy(() => {
    if (documentData) {
      const cleanup = setup_auto_save(documentData, () => {});
      if (cleanup) cleanup();
    }
  });

  // Toggle cursor prompt visibility
  function toggleCursorPrompt() {
    showCursorPrompt = !showCursorPrompt;
  }
</script>

<svelte:head>
  <title>{documentData ? documentData.name : 'Document'} | Vynn</title>
</svelte:head>

<div class="editor-page">
  <div class="background-image" style="background-image: url({backgroundImage})"></div>
  
  <!-- Minimal Navbar -->
  <nav class="navbar">
    <a href="/drive" class="logo-link" aria-label="Go to Drive">
      <div class="logo-container">
        <img src={logo} alt="Vynn Logo" class="logo" />
        <span class="logo-text">Vynn</span>
      </div>
    </a>
    <div class="spacer"></div>
    <a href="/profile" class="profile-link" aria-label="Go to Profile">
      <div class="profile-image"></div>
    </a>
  </nav>
  
  <!-- Project Document Switcher -->
  <div class="document-switcher">
    {#if projectDocuments.length > 0}
      {#each projectDocuments as doc, i}
        <button 
          class="doc-button {doc.id.toString() === documentId ? 'active' : ''}" 
          on:click={() => switchDocument(doc.id)}
          aria-label="Switch to {doc.name}"
          title="{doc.name} (ID: {doc.id})"
        >
          {i + 1}
        </button>
      {/each}
    {:else}
      <!-- Show a single button for standalone documents -->
      <button 
        class="doc-button active" 
        aria-label="Current document"
      >
        1
      </button>
    {/if}
  </div>
  
  <!-- Editor Container with animation -->
  <div class="editor-container">
    {#if loading}
      <div class="loading">Loading document...</div>
    {:else if error}
      <div class="error">Error loading document</div>
    {:else}
      <!-- Previous document (for animation) -->
      {#if isAnimating && previousDocumentContent}
        <div class="editor-wrapper previous {slideDirection}-exit" style={animationHeight ? `height: ${animationHeight}px` : ''}>
          <div class="editor-content">
            <div class="line-numbers">
              {#each previousDocumentLines as line, i}
                <div class="line-number {i === previousActiveLineIndex ? 'active' : ''}">{i + 1}</div>
              {/each}
            </div>
            <div class="editor-textarea-static">{previousDocumentContent}</div>
          </div>
        </div>
      {/if}
      
      <!-- Current document -->
      <div class="editor-wrapper current {isAnimating ? `${slideDirection}-enter` : ''}" style={animationHeight ? `height: ${animationHeight}px` : ''}>
        <div class="editor-content">
          <div class="line-numbers">
            {#each lines as line, i}
              <div class="line-number {i === activeLineIndex ? 'active' : ''}">{i + 1}</div>
            {/each}
          </div>
          <textarea
            bind:this={editorElement}
            bind:value={editorContent}
            on:keydown={handleKeyDown}
            on:input={handleInput}
            on:click={handleInput}
            on:keyup={handleInput}
            class="editor-textarea"
            spellcheck="false"
            autocomplete="off"
            {...{ autocorrect: "off" } as any}
            autocapitalize="off"
          ></textarea>
        </div>
      </div>
    {/if}
  </div>
  
  <!-- Status Bar with Cursor Prompts Toggle -->
  <div class="status-bar">
    <div class="mode-indicator">{editorMode}</div>
    <div class="cursor-position">Line {cursorLine}, Col {cursorColumn}</div>
    <button class="prompt-toggle" on:click={toggleCursorPrompt}>
      {showCursorPrompt ? 'Hide' : 'Show'} Prompts
    </button>
  </div>
  
  <!-- Cursor Prompts Panel -->
  {#if showCursorPrompt}
    <div class="cursor-prompts">
      <div class="cursor-prompts-header">
        <h4>{editorMode} Mode Commands</h4>
      </div>
      <div class="cursor-prompts-list">
        {#each cursorPrompts[editorMode] as prompt}
          <div class="prompt-item">
            <span class="key">{prompt.key}</span>
            <span class="description">{prompt.description}</span>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .editor-page {
    min-height: 100vh;
    width: 100%;
    display: flex;
    flex-direction: column;
    color: #E5E5E5;
    position: relative;
    overflow-y: auto; /* Allow vertical scrolling */
    padding-bottom: 50px; /* Add space for the fixed status bar */
  }
  
  .background-image {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-size: cover;
    background-position: center;
    filter: blur(8px);
    transform: scale(1.1);
    z-index: -1;
  }
  
  .navbar {
    display: flex;
    align-items: center;
    padding: 0.5rem 1rem;
    background-color: rgba(10, 23, 33, 0.7);
    backdrop-filter: blur(5px);
    height: 60px;
    position: relative;
    z-index: 2; /* Higher z-index */
    border-bottom: 1px solid rgba(16, 185, 129, 0.3);
  }
  
  .logo-container {
    display: flex;
    align-items: center;
  }
  
  .logo {
    height: 40px;
    width: auto;
  }
  
  .logo-link {
    text-decoration: none;
  }
  
  .logo-text {
    margin-left: 10px;
    font-size: 24px;
    font-weight: bold;
    color: #E5E5E5;
    font-family: 'JetBrains Mono', monospace;
  }
  
  .spacer {
    flex-grow: 1;
  }
  
  .profile-image {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background-color: #555;
    border: 2px solid #10B981;
  }
  
  .editor-container {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding: 2rem 2rem 0 2rem;
    position: relative;
    z-index: 1;
    min-height: 75vh;
    overflow-y: auto; /* Allow vertical scrolling */
    max-height: calc(100vh - 150px); /* Limit height to prevent overflow */
  }
  
  .editor-wrapper {
    position: absolute;
    width: 90%;
    max-width: 1400px;
    background-color: rgba(10, 23, 33, 0.7);
    backdrop-filter: blur(5px);
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    border: 1px solid rgba(16, 185, 129, 0.3);
    box-shadow: 0 0 20px rgba(16, 185, 129, 0.2);
    transition: transform 0.3s ease, opacity 0.3s ease;
    left: 0;
    right: 0;
    margin: 0 auto;
  }
  
  .editor-wrapper.current {
    z-index: 1;
  }
  
  .editor-wrapper.previous {
    z-index: 0;
  }
  
  /* Exit animations */
  .editor-wrapper.left-exit {
    animation: slideOutLeft 0.3s forwards;
  }
  
  .editor-wrapper.right-exit {
    animation: slideOutRight 0.3s forwards;
  }
  
  /* Enter animations */
  .editor-wrapper.left-enter {
    animation: slideInRight 0.3s forwards;
  }
  
  .editor-wrapper.right-enter {
    animation: slideInLeft 0.3s forwards;
  }
  
  .editor-content {
    display: flex;
    flex: 1;
    overflow: visible;
    min-height: 100%;
  }
  
  .line-numbers {
    padding: 1.5rem 0.5rem 1.5rem 1rem;
    background-color: transparent;
    font-family: 'JetBrains Mono', monospace;
    font-size: 16px;
    line-height: 1.5;
    color: rgba(229, 229, 229, 0.5);
    text-align: right;
    min-width: 3rem;
    user-select: none;
    position: relative;
    height: auto;
  }
  
  .line-numbers::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(10, 23, 33, 0.7);
    filter: blur(8px);
    z-index: -1;
  }
  
  .line-number {
    height: 1.5rem;
    position: relative;
    z-index: 1;
  }
  
  .line-number.active {
    color: #10B981;
    font-weight: bold;
  }
  
  .editor-textarea {
    flex: 1;
    background-color: transparent;
    color: #E5E5E5;
    border: none;
    resize: none;
    padding: 1.5rem;
    font-family: 'JetBrains Mono', monospace;
    font-size: 16px;
    line-height: 1.5;
    outline: none;
    overflow-y: auto; /* Allow vertical scrolling */
    min-height: 100%;
  }
  
  .editor-textarea-static {
    flex: 1;
    background-color: transparent;
    color: #E5E5E5;
    padding: 1.5rem;
    font-family: 'JetBrains Mono', monospace;
    font-size: 16px;
    line-height: 1.5;
    white-space: pre-wrap;
    overflow-y: hidden;
    min-height: 100%;
  }
  
  .status-bar {
    display: flex;
    justify-content: space-between;
    padding: 0.5rem 1rem;
    font-family: 'JetBrains Mono', monospace;
    font-size: 14px;
    position: fixed; /* Make it fixed */
    bottom: 0; /* Position at bottom */
    left: 0;
    right: 0;
    z-index: 10; /* Ensure it's above other content */
    background-color: rgba(10, 23, 33, 0.9);
    backdrop-filter: blur(5px);
    border-top: 1px solid rgba(16, 185, 129, 0.3);
  }
  
  .mode-indicator {
    font-weight: bold;
    color: #10B981;
  }
  
  .loading, .error {
    color: #E5E5E5;
    font-size: 18px;
    text-align: center;
  }
  
  .error {
    color: #EF4444;
  }
  
  /* Update document switcher styles */
  .document-switcher {
    display: flex;
    justify-content: center;
    gap: 10px;
    padding: 10px 0;
    margin-top: 30px;
    position: relative; /* Add position relative */
    z-index: 1; /* Lower z-index than navbar */
  }
  
  .doc-button {
    width: 50px;
    height: 30px;
    border-radius: 35%;
    background-color: rgba(16, 185, 129, 0.2);
    border: 1px solid rgba(16, 185, 129, 0.5);
    color: #E5E5E5;
    font-family: 'JetBrains Mono', monospace;
    font-size: 14px;
    font-weight: bold;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .doc-button:hover {
    background-color: rgba(16, 185, 129, 0.4);
    transform: scale(1.05);
  }
  
  .doc-button.active {
    background-color: rgba(16, 185, 129, 0.6);
    border: 2px solid #10B981;
  }
  
  /* Adjust editor container to account for document switcher */
  .editor-container {
    padding-top: 1rem;
  }
  
  @keyframes slideOutLeft {
    0% {
      transform: translateX(0);
      opacity: 1;
    }
    100% {
      transform: translateX(-100%);
      opacity: 0;
    }
  }
  
  @keyframes slideOutRight {
    0% {
      transform: translateX(0);
      opacity: 1;
    }
    100% {
      transform: translateX(100%);
      opacity: 0;
    }
  }
  
  @keyframes slideInRight {
    0% {
      transform: translateX(100%);
      opacity: 0;
    }
    100% {
      transform: translateX(0);
      opacity: 1;
    }
  }
  
  @keyframes slideInLeft {
    0% {
      transform: translateX(-100%);
      opacity: 0;
    }
    100% {
      transform: translateX(0);
      opacity: 1;
    }
  }
  
  /* Cursor Prompts Styles */
  .cursor-prompts {
    position: fixed;
    right: 20px;
    bottom: 50px;
    width: 300px;
    background-color: rgba(10, 23, 33, 0.9);
    backdrop-filter: blur(10px);
    border-radius: 8px;
    border: 1px solid rgba(16, 185, 129, 0.4);
    box-shadow: 0 0 15px rgba(16, 185, 129, 0.2);
    z-index: 100;
    overflow: hidden;
    transition: opacity 0.3s ease, transform 0.3s ease;
  }
  
  .cursor-prompts-header {
    padding: 10px 15px;
    background-color: rgba(16, 185, 129, 0.2);
    border-bottom: 1px solid rgba(16, 185, 129, 0.4);
  }
  
  .cursor-prompts-header h4 {
    margin: 0;
    color: #10B981;
    font-size: 16px;
    font-weight: bold;
  }
  
  .cursor-prompts-list {
    max-height: 300px;
    overflow-y: auto;
    padding: 10px 0;
  }
  
  .prompt-item {
    display: flex;
    padding: 8px 15px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }
  
  .prompt-item:last-child {
    border-bottom: none;
  }
  
  .prompt-item .key {
    display: inline-block;
    min-width: 50px;
    padding: 2px 8px;
    background-color: rgba(16, 185, 129, 0.2);
    border: 1px solid rgba(16, 185, 129, 0.4);
    border-radius: 4px;
    color: #10B981;
    font-family: 'JetBrains Mono', monospace;
    font-size: 12px;
    font-weight: bold;
    text-align: center;
    margin-right: 10px;
  }
  
  .prompt-item .description {
    color: #E5E5E5;
    font-size: 14px;
    display: flex;
    align-items: center;
  }
  
  .prompt-toggle {
    background-color: rgba(16, 185, 129, 0.2);
    border: 1px solid rgba(16, 185, 129, 0.4);
    border-radius: 4px;
    color: #10B981;
    font-size: 12px;
    padding: 2px 8px;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .prompt-toggle:hover {
    background-color: rgba(16, 185, 129, 0.4);
  }
</style> 