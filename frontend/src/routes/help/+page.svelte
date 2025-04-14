<script lang="ts">
    import { onMount } from 'svelte';
    import { check_auth } from '$lib/ts/user';
    import Navbar from '$lib/components/Navbar.svelte';
    import Footer from '$lib/components/Footer.svelte';
    
    let isLoggedIn = false;
    let isSubmitting = false;
    let successMessage = '';
    let errorMessage = '';
    
    // Form data
    let name = '';
    let email = '';
    let subject = '';
    let message = '';
    
    onMount(async () => {
        try {
            isLoggedIn = await check_auth();
        } catch (error) {
            console.error('Error checking authentication:', error);
        }
    });
    
    async function handleSubmit() {
        try {
            isSubmitting = true;
            errorMessage = '';
            successMessage = '';
            
            // Validate form
            if (!name.trim() || !email.trim() || !message.trim()) {
                errorMessage = 'Please fill out all required fields';
                return;
            }
            
            // For now, we'll simulate sending the support request
            // In a real implementation, you would call an API endpoint
            await new Promise(resolve => setTimeout(resolve, 1000));
            
            // Clear form data
            name = '';
            email = '';
            subject = '';
            message = '';
            
            // Show success message
            successMessage = 'Your support request has been submitted. We will contact you soon.';
            
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
    <Navbar {isLoggedIn} />
    
    <div class="container py-5">
        <div class="row justify-content-center mb-5">
            <div class="col-lg-8 text-center">
                <h1 class="fw-bold mb-4">Help & Support</h1>
                <p class="fs-5 text-white-50 mb-5">
                    Need assistance with Vynn? Our support team is here to help.
                    Fill out the form below, and we'll get back to you as soon as possible.
                </p>
            </div>
        </div>
        
