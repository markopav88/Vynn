<script lang="ts">
    import { onMount } from 'svelte';
    import { check_auth } from '$lib/ts/user';
    import { get_current_user } from '$lib/ts/user';
    import logo from '$lib/assets/logo.png';
    import Navbar from '$lib/components/Navbar.svelte';
    import Footer from '$lib/components/Footer.svelte';
    import '$lib/assets/style/tutorial.css';

    let isLoggedIn = false;
    let pageLoaded = false;
    let currentStep = 1;
    let totalSteps = 8;
    let redirectPath = '/login';

    onMount(async () => {
        document.title = "Vynn - Tutorial";
        try {
            const user = await get_current_user();
            isLoggedIn = !!user;
            redirectPath = user ? '/drive' : '/login';
        } catch (error) {
            console.error('Error checking authentication:', error);
            redirectPath = '/login';
        }

        setTimeout(() => {
            pageLoaded = true;
        }, 100);
    });

    function nextStep() {
        if (currentStep < totalSteps) {
            currentStep++;
        }
    }

    function prevStep() {
        if (currentStep > 1) {
            currentStep--;
        }
    }
</script>

<div class="bg-black min-vh-100 d-flex flex-column">
    <Navbar {isLoggedIn} />

    <div class="container flex-grow-1 py-5">
        <!-- Tutorial Header -->
        <div class="text-center mb-5 fade-in {pageLoaded ? 'visible' : ''}">
            <h1 class="display-4 fw-bold mb-4">Getting Started with Vynn</h1>
            <p class="text-white-50 fs-5 mb-5">Learn how to use Vynn's powerful features in just a few minutes.</p>
        </div>

        <!-- Tutorial Progress -->
        <div class="progress bg-dark mb-5 fade-in {pageLoaded ? 'visible' : ''}" style="height: 8px;">
            <div
                class="progress-bar bg-green"
                role="progressbar"
                style="width: {(currentStep / totalSteps) * 100}%"
                aria-valuenow={currentStep}
                aria-valuemin="0"
                aria-valuemax={totalSteps}
            ></div>
        </div>

        <!-- Tutorial Content -->
        <div class="row justify-content-center">
            <div class="col-lg-10">
                <div class="card bg-dark border-0 shadow-lg fade-in {pageLoaded ? 'visible' : ''}">
                    <div class="card-body p-4 p-md-5">
                        <!-- Step Content -->
                        {#if currentStep === 1}
                            <div class="text-center mb-4">
                                <h2 class="fw-bold mb-3">Welcome to Vynn</h2>
                                <p class="text-white-50">
                                    Vynn combines the power of Vim with AI assistance to enhance your writing experience.
                                    Let's walk through the key features that make Vynn unique.
                                </p>
                                <img src={logo} alt="Vynn Logo" class="mt-4" width="80" height="80" />
                            </div>
                        {:else if currentStep === 2}
                            <div class="text-center mb-4">
                                <h2 class="fw-bold mb-3">Editing Modes</h2>
                                <p class="text-white-50">
                                    Vynn uses different modes for efficient text editing. Each mode serves a specific purpose,
                                    making your editing experience more powerful and precise.
                                </p>
                                <div class="modes-preview mt-4">
                                    <div class="mode-item">
                                        <kbd>Esc</kbd>
                                        <h5 class="text-green mt-2">Normal Mode</h5>
                                        <p class="small text-white-50">Navigate and manipulate text with powerful commands</p>
                                    </div>
                                    <div class="mode-item">
                                        <kbd>i</kbd>
                                        <h5 class="text-green mt-2">Insert Mode</h5>
                                        <p class="small text-white-50">Type and edit text as in regular editors</p>
                                    </div>
                                    <div class="mode-item">
                                        <kbd>/</kbd>
                                        <h5 class="text-green mt-2">Forward Search</h5>
                                        <p class="small text-white-50">Search text forward from cursor position</p>
                                    </div>
                                    <div class="mode-item">
                                        <kbd>?</kbd>
                                        <h5 class="text-green mt-2">Backward Search</h5>
                                        <p class="small text-white-50">Search text backward from cursor position</p>
                                    </div>
                                    <div class="mode-item">
                                        <kbd>:</kbd>
                                        <h5 class="text-green mt-2">Command Mode</h5>
                                        <p class="small text-white-50">Execute commands and access advanced features</p>
                                    </div>
                                    <div class="mode-item">
                                        <kbd>diff</kbd>
                                        <h5 class="text-green mt-2">Diff Mode</h5>
                                        <p class="small text-white-50">View AI suggestions in read-only mode for reviewing changes</p>
                                    </div>
                                </div>
                            </div>
                        {:else if currentStep === 3}
                            <div class="text-center mb-4">
                                <h2 class="fw-bold mb-3">Documents & Projects</h2>
                                <p class="text-white-50">
                                    Organize your work efficiently with Projects and Documents. Group related documents
                                    together to create context windows, helping AI better understand your content.
                                </p>
                                <div class="features-preview mt-4">
                                    <div class="feature-item">
                                        <i class="bi bi-folder-fill text-green mb-2"></i>
                                        <p class="small text-white-50">Create and manage Projects</p>
                                    </div>
                                    <div class="feature-item">
                                        <i class="bi bi-file-text-fill text-green mb-2"></i>
                                        <p class="small text-white-50">Add Documents to Projects</p>
                                    </div>
                                    <div class="feature-item">
                                        <i class="bi bi-share text-green mb-2"></i>
                                        <p class="small text-white-50">Star, Delete, and Share with others</p>
                                    </div>
                                </div>
                            </div>
                        {:else if currentStep === 4}
                            <div class="text-center mb-4">
                                <h2 class="fw-bold mb-3">Command Cheat Sheet</h2>
                                <p class="text-white-50">
                                    Access Vim commands instantly with <kbd>Ctrl</kbd> + <kbd>/</kbd>. View and search all available
                                    commands, which can be customized in your profile settings.
                                </p>
                                <div class="terminal-preview mt-4">
                                    <div class="d-flex justify-content-between align-items-center mb-2">
                                        <span class="text-white-50">Command Cheat Sheet</span>
                                        <kbd>Ctrl + /</kbd>
                                    </div>
                                    <pre class="text-green">ctrl + u - Underline text
d - Delete line
G - Go to end of document
:w - Save file</pre>
                                </div>
                            </div>
                        {:else if currentStep === 5}
                            <div class="text-center mb-4">
                                <h2 class="fw-bold mb-3">AI Chat Assistant</h2>
                                <p class="text-white-50">
                                    Open the AI chat panel with <kbd>Alt</kbd> + <kbd>C</kbd> to get instant help.
                                    Free users get 10 prompts to start with. Upgrade for unlimited access.
                                </p>
                                <div class="ai-preview mt-4">
                                    <div class="chat-preview">
                                        <div class="chat-message">How can I help you today?</div>
                                        <div class="prompts-left">9 prompts remaining</div>
                                    </div>
                                </div>
                            </div>
                        {:else if currentStep === 6}
                            <div class="text-center mb-4">
                                <h2 class="fw-bold mb-3">AI Commands</h2>
                                <p class="text-white-50">
                                    Execute powerful AI commands directly in your editor. Use commands like
                                    <code>:grammar</code> for instant writing improvements.
                                </p>
                                <div class="terminal-preview mt-4">
                                    <pre class="text-green">:grammar - Check grammar
:style - Improve writing style
:summarize - Summarize text
:explain - Explain selection</pre>
                                </div>
                            </div>
                        {:else if currentStep === 7}
                            <div class="text-center mb-4">
                                <h2 class="fw-bold mb-3">AI Suggestions</h2>
                                <p class="text-white-50">
                                    Review AI suggestions in an intuitive diff format. Accept or reject changes
                                    with a single click, maintaining full control over your content.
                                </p>
                                <div class="diff-preview mt-4">
                                    <div class="diff-line deletion">- The meeting was very productive.</div>
                                    <div class="diff-line addition">+ The meeting yielded significant results.</div>
                                    <div class="diff-actions">
                                        <button class="btn btn-sm btn-outline-success">Accept</button>
                                        <button class="btn btn-sm btn-outline-danger">Reject</button>
                                    </div>
                                </div>
                            </div>
                        {:else if currentStep === 8}
                            <div class="text-center mb-4">
                                <h2 class="fw-bold mb-3">Customize Your Editor</h2>
                                <p class="text-white-50">
                                    Make Vynn yours by customizing the editor environment through the Profile page's
                                    User Preferences tab. Choose themes, fonts, and layout options.
                                </p>
                                <div class="preferences-preview mt-4">
                                    <div class="theme-samples">
                                        <div class="theme-sample dark-theme">Dark</div>
                                        <div class="theme-sample light-theme">Light</div>
                                        <div class="theme-sample custom-theme">Custom</div>
                                    </div>
                                </div>
                            </div>
                        {/if}

                        <!-- Navigation Buttons -->
                        <div class="d-flex justify-content-between mt-5">
                            <button
                                class="btn btn-outline-light px-4"
                                on:click={prevStep}
                                disabled={currentStep === 1}
                            >
                                <i class="bi bi-arrow-left me-2"></i>
                                Previous
                            </button>
                            <div class="d-flex align-items-center">
                                <span class="text-white-50">Step {currentStep} of {totalSteps}</span>
                            </div>
                            {#if currentStep < totalSteps}
                                <button class="btn btn-green px-4" on:click={nextStep}>
                                    Next
                                    <i class="bi bi-arrow-right ms-2"></i>
                                </button>
                            {:else}
                                <a href={redirectPath} class="btn btn-green px-4">
                                    {isLoggedIn ? 'Go to Drive' : 'Login to Start'}
                                    <i class="bi bi-arrow-right ms-2"></i>
                                </a>
                            {/if}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <Footer />
</div> 