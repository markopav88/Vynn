<script lang="ts">
    import { onMount } from 'svelte';
    import { check_auth } from '$lib/ts/user';
    import Navbar from '$lib/components/Navbar.svelte';
    import Footer from '$lib/components/Footer.svelte';
    
    let isLoggedIn = false;
    let isSubmitting = false;
    let successMessage = '';
    let errorMessage = '';
    let isAuthChecked = false;
    
    // Form data
    let name = '';
    let email = '';
    let subject = '';
    let message = '';
    
    onMount(async () => {
        document.title = "Vynn - Help";
        try {
            // Complete the auth check first
            isLoggedIn = await check_auth();
            // Mark auth as checked to render the Navbar
            isAuthChecked = true;
            
            // Make sure the page is properly positioned
            // This helps ensure the hero section is visible below the navbar
            window.scrollTo(0, 0);
            
            // Ensure Bootstrap is initialized when navigating directly to this page
            if (typeof window !== 'undefined') {
                // Ensure the accordion is properly initialized after navigation
                setTimeout(() => {
                    // This timeout ensures Bootstrap has time to load
                    if (window.bootstrap && window.bootstrap.Collapse) {
                        document.querySelectorAll('.accordion-collapse').forEach(el => {
                            new window.bootstrap.Collapse(el, { toggle: false });
                        });
                    }
                }, 100);
            }
        } catch (error) {
            console.error('Error checking authentication:', error);
            isAuthChecked = true; // Still render the page even if auth check fails
        }
    });
    
    async function handleSubmit(event: SubmitEvent) {
        try {
            event.preventDefault();
            isSubmitting = true;
            errorMessage = '';
            successMessage = '';
            
            // Validate form
            if (!name.trim() || !email.trim() || !message.trim()) {
                errorMessage = 'Please fill out all required fields';
                isSubmitting = false;
                return;
            }
            
            // Create FormData object
            const formData = new FormData();
            formData.append('name', name);
            formData.append('email', email);
            formData.append('subject', subject);
            formData.append('message', message);
            
            // Send to Formspree
            const response = await fetch("https://formspree.io/f/mldrywqe", {
                method: "POST",
                body: formData,
                headers: {
                    Accept: "application/json",
                }
            });
            
            if (response.ok) {
                // Clear form data
                name = '';
                email = '';
                subject = '';
                message = '';
                
                // Show success message
                successMessage = 'Your support request has been submitted. We will contact you soon.';
                
                // Reset form after a delay if needed
                setTimeout(() => {
                    successMessage = '';
                }, 5000);
            } else {
                // Handle error response
                const errorData = await response.json();
                errorMessage = errorData.error || 'Failed to submit the form. Please try again.';
            }
            
        } catch (error) {
            console.error('Error submitting support request:', error);
            errorMessage = 'An unexpected error occurred. Please try again.';
        } finally {
            isSubmitting = false;
        }
    }
</script>

<svelte:head>
    <title>Help & Support | Vynn</title>
</svelte:head>

<div class="bg-black min-vh-100 d-flex flex-column">
    {#if isAuthChecked}
        <Navbar {isLoggedIn} />
        
        <div class="container py-5">
            <div class="row justify-content-center mb-5">
                <div class="col-lg-8 text-center mt-20">
                    <h1 class="fw-bold mb-4">Help & Support</h1>
                    <p class="fs-5 text-white-50 mb-5">
                        Need assistance with Vynn? Our support team is here to help.
                        Fill out the form below, and we'll get back to you as soon as possible.
                    </p>
                </div>
            </div>
            
            <div class="row">
                <!-- Left column: FAQ -->
                <div class="col-lg-5 mb-5 mb-lg-0">
                    <div class="card bg-dark text-white border-0 shadow">
                        <div class="card-body p-4">
                            <h2 class="card-title mb-4">Frequently Asked Questions</h2>
                            
                            <div class="accordion accordion-dark" id="faqAccordion">
                                <!-- FAQ Item 1 -->
                                <div class="accordion-item bg-dark border-secondary">
                                    <h3 class="accordion-header">
                                        <button 
                                            class="accordion-button bg-dark text-white collapsed" 
                                            type="button" 
                                            data-bs-toggle="collapse" 
                                            data-bs-target="#faq1"
                                        >
                                            How do I customize keybindings?
                                        </button>
                                    </h3>
                                    <div id="faq1" class="accordion-collapse collapse" data-bs-parent="#faqAccordion">
                                        <div class="accordion-body text-white-50">
                                            You can customize your keybindings in your Account settings. Go to your profile and find the "Keybindings" tab to set your preferred shortcuts.
                                        </div>
                                    </div>
                                </div>
                                
                                <!-- FAQ Item 2 -->
                                <div class="accordion-item bg-dark border-secondary">
                                    <h3 class="accordion-header">
                                        <button 
                                            class="accordion-button bg-dark text-white collapsed" 
                                            type="button" 
                                            data-bs-toggle="collapse" 
                                            data-bs-target="#faq2"
                                        >
                                            Can I export my documents?
                                        </button>
                                    </h3>
                                    <div id="faq2" class="accordion-collapse collapse" data-bs-parent="#faqAccordion">
                                        <div class="accordion-body text-white-50">
                                            Yes! When viewing a document, click the "Export PDF" button in the document toolbar to download your document as a PDF file.
                                        </div>
                                    </div>
                                </div>
                                
                                <!-- FAQ Item 3 -->
                                <div class="accordion-item bg-dark border-secondary">
                                    <h3 class="accordion-header">
                                        <button 
                                            class="accordion-button bg-dark text-white collapsed" 
                                            type="button" 
                                            data-bs-toggle="collapse" 
                                            data-bs-target="#faq3"
                                        >
                                            How do I use Vim commands?
                                        </button>
                                    </h3>
                                    <div id="faq3" class="accordion-collapse collapse" data-bs-parent="#faqAccordion">
                                        <div class="accordion-body text-white-50">
                                            In the document editor, press <kbd>Ctrl</kbd>+<kbd>/</kbd> to toggle the Vim commands reference sheet. You can also check our tutorial page for a comprehensive guide.
                                        </div>
                                    </div>
                                </div>
                                
                                <!-- FAQ Item 4 -->
                                <div class="accordion-item bg-dark border-secondary">
                                    <h3 class="accordion-header">
                                        <button 
                                            class="accordion-button bg-dark text-white collapsed" 
                                            type="button" 
                                            data-bs-toggle="collapse" 
                                            data-bs-target="#faq4"
                                        >
                                            How can I share documents?
                                        </button>
                                    </h3>
                                    <div id="faq4" class="accordion-collapse collapse" data-bs-parent="#faqAccordion">
                                        <div class="accordion-body text-white-50">
                                            To share a document, go to your Drive, select the document you want to share, and click on "Share". You can then enter the email address of the person you want to share it with and set their permission level.
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                    
                    <!-- Additional Help Resources -->
                    <div class="card bg-dark text-white border-0 shadow mt-4">
                        <div class="card-body p-4">
                            <h3 class="card-title mb-3">Additional Resources</h3>
                            
                            <div class="list-group list-group-flush bg-dark">
                                <a href="/tutorial" class="list-group-item list-group-item-action bg-dark text-white border-secondary">
                                    <i class="bi bi-mortarboard-fill me-2 text-green"></i> Tutorial
                                </a>
                                <a href="https://github.com/MaristGormanly/something" target="_blank" class="list-group-item list-group-item-action bg-dark text-white border-secondary">
                                    <i class="bi bi-github me-2 text-green"></i> GitHub Repository
                                </a>
                                <a href="mailto:support@vynn.example.com" class="list-group-item list-group-item-action bg-dark text-white border-secondary">
                                    <i class="bi bi-envelope-fill me-2 text-green"></i> Email Support
                                </a>
                            </div>
                        </div>
                    </div>
                </div>
                
                <!-- Right column: Support Form -->
                <div class="col-lg-7">
                    <div class="card bg-dark text-white border-0 shadow">
                        <div class="card-body p-4">
                            <h2 class="card-title mb-4">Contact Support</h2>
                            
                            {#if successMessage}
                                <div class="alert alert-success mb-4" role="alert">
                                    {successMessage}
                                </div>
                            {/if}
                            
                            {#if errorMessage}
                                <div class="alert alert-danger mb-4" role="alert">
                                    {errorMessage}
                                </div>
                            {/if}
                            
                            <form on:submit|preventDefault={handleSubmit}>
                                <!-- Name -->
                                <div class="mb-3">
                                    <label for="name" class="form-label">Name <span class="text-danger">*</span></label>
                                    <input 
                                        type="text" 
                                        class="form-control bg-black text-white border-secondary" 
                                        id="name" 
                                        bind:value={name} 
                                        required
                                        placeholder="Your name"
                                    />
                                </div>
                                
                                <!-- Email -->
                                <div class="mb-3">
                                    <label for="email" class="form-label">Email <span class="text-danger">*</span></label>
                                    <input 
                                        type="email" 
                                        class="form-control bg-black text-white border-secondary" 
                                        id="email" 
                                        bind:value={email} 
                                        required
                                        placeholder="Your email address"
                                    />
                                </div>
                                
                                <!-- Subject -->
                                <div class="mb-3">
                                    <label for="subject" class="form-label">Subject</label>
                                    <input 
                                        type="text" 
                                        class="form-control bg-black text-white border-secondary" 
                                        id="subject" 
                                        bind:value={subject} 
                                        placeholder="What is your question about?"
                                    />
                                </div>
                                
                                <!-- Message -->
                                <div class="mb-4">
                                    <label for="message" class="form-label">Message <span class="text-danger">*</span></label>
                                    <textarea 
                                        class="form-control bg-black text-white border-secondary" 
                                        id="message" 
                                        bind:value={message} 
                                        rows="6" 
                                        required
                                        placeholder="Please describe your issue or question in detail"
                                    ></textarea>
                                </div>
                                
                                <!-- Submit Button -->
                                <button 
                                    type="submit" 
                                    class="btn btn-green btn-lg w-100" 
                                    disabled={isSubmitting}
                                >
                                    {#if isSubmitting}
                                        <span class="spinner-border spinner-border-sm me-2" role="status" aria-hidden="true"></span>
                                        Submitting...
                                    {:else}
                                        Submit Support Request
                                    {/if}
                                </button>
                            </form>
                        </div>
                    </div>
                </div>
            </div>
        </div>
        
        <Footer />
    {:else}
        <!-- Show loading spinner while auth check completes -->
        <div class="vh-100 d-flex justify-content-center align-items-center">
            <div class="spinner-border text-green" role="status">
                <span class="visually-hidden">Loading...</span>
            </div>
        </div>
    {/if}
</div>

<style>
    /* Customize accordion styles */
    :global(.accordion-button:not(.collapsed)) {
        background-color: rgba(16, 185, 129, 0.1) !important;
        color: var(--color-primary) !important;
        box-shadow: none !important;
    }
    
    :global(.accordion-button:focus) {
        box-shadow: none !important;
        border-color: rgba(16, 185, 129, 0.5) !important;
    }
    
    :global(.accordion-button::after) {
        filter: invert(1);
    }
    
    /* Style for keyboard keys */
    kbd {
        background-color: #333;
        border: 1px solid #666;
        border-radius: 3px;
        color: #fff;
        display: inline-block;
        font-size: 0.9em;
        padding: 2px 5px;
    }

    /* Fix for hero element positioning */
    :global(.container.py-5) {
        padding-top: 7rem !important; /* Ensure space for navbar */
        transition: padding-top 0.2s ease;
    }

    /* Ensure proper page section spacing */
    :global(.row.justify-content-center.mb-5) {
        margin-top: 1rem;
    }
</style> 