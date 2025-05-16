<script lang="ts">
    import { onMount } from 'svelte';
    import Navbar from '$lib/components/Navbar.svelte';
    import Footer from '$lib/components/Footer.svelte';
    import { get_current_user } from '$lib/ts/user';
    import '$lib/assets/style/plans.css';
    
    let isLoggedIn = false;
    let pageLoaded = false;

    onMount(async () => {
        document.title = "Vynn - Plans";
        try {
            const user = await get_current_user();
            isLoggedIn = !!user;
        } catch (error) {
            console.error('Error checking authentication:', error);
        }
        setTimeout(() => {
            pageLoaded = true;
        }, 300);
    });
</script>

<div class="bg-black min-vh-100 d-flex flex-column">
    <Navbar {isLoggedIn} />

    <div class="container flex-grow-1 py-5">
        <!-- Pricing Header -->
        <div class="text-center mb-5">
            <h1 class="display-4 fw-bold mb-4">Simple, Transparent Pricing</h1>
            <p class="text-white-50 fs-5">Choose the plan that works best for you</p>
        </div>

        <!-- Pricing Cards -->
        <div class="row g-4 justify-content-center mb-5">
            <!-- Starter Plan -->
            <div class="col-md-4">
                <div class="card h-100 bg-dark text-white border-0">
                    <div class="card-body p-4">
                        <h3 class="card-title fw-bold mb-4">Starter</h3>
                        <p class="text-white-50 mb-4">Perfect for individual writers</p>

                        <div class="mb-4">
                            <h2 class="display-5 fw-bold mb-0">$9</h2>
                            <p class="text-white-50">/month</p>
                        </div>

                        <ul class="list-unstyled mb-4">
                            <li class="mb-2">
                                <i class="bi bi-check-circle-fill text-green me-2"></i>
                                Basic AI assistance
                            </li>
                            <li class="mb-2">
                                <i class="bi bi-check-circle-fill text-green me-2"></i>
                                5 projects
                            </li>
                            <li class="mb-2">
                                <i class="bi bi-check-circle-fill text-green me-2"></i>
                                Standard support
                            </li>
                            <li class="mb-2">
                                <i class="bi bi-check-circle-fill text-green me-2"></i>
                                Unlimited Documents
                            </li>
                        </ul>

                        <button class="btn btn-outline-light w-100" disabled>Get Started</button>
                    </div>
                </div>
            </div>

            <!-- Pro Plan -->
            <div class="col-md-4">
                <div class="card h-100 bg-dark text-white border-0 featured-card">
                    <div class="position-absolute top-0 start-50 translate-middle">
                        <span class="badge bg-green px-4 py-2 rounded-pill mt-2 popular-badge">Popular</span>
                    </div>
                    <div class="card-body p-4">
                        <h3 class="card-title fw-bold mb-4">Pro</h3>
                        <p class="text-white-50 mb-4">For professional writers</p>

                        <div class="mb-4">
                            <h2 class="display-5 fw-bold mb-0">$19</h2>
                            <p class="text-white-50">/month</p>
                        </div>

                        <ul class="list-unstyled mb-4">
                            <li class="mb-2">
                                <i class="bi bi-check-circle-fill text-green me-2"></i>
                                Advanced AI features
                            </li>
                            <li class="mb-2">
                                <i class="bi bi-check-circle-fill text-green me-2"></i>
                                Unlimited projects
                            </li>
                            <li class="mb-2">
                                <i class="bi bi-check-circle-fill text-green me-2"></i>
                                All themes
                            </li>
                            <li class="mb-2">
                                <i class="bi bi-check-circle-fill text-green me-2"></i>
                                Priority support
                            </li>
                        </ul>

                        <button class="btn btn-green w-100" disabled>Get Started</button>
                    </div>
                </div>
            </div>

            <!-- Team Plan -->
            <div class="col-md-4">
                <div class="card h-100 bg-dark text-white border-0">
                    <div class="card-body p-4">
                        <h3 class="card-title fw-bold mb-4">Team</h3>
                        <p class="text-white-50 mb-4">For writing teams</p>

                        <div class="mb-4">
                            <h2 class="display-5 fw-bold mb-0">$49</h2>
                            <p class="text-white-50">/month</p>
                        </div>

                        <ul class="list-unstyled mb-4">
                            <li class="mb-2">
                                <i class="bi bi-check-circle-fill text-green me-2"></i>
                                Everything in Pro
                            </li>
                            <li class="mb-2">
                                <i class="bi bi-check-circle-fill text-green me-2"></i>
                                Team collaboration
                            </li>
                            <li class="mb-2">
                                <i class="bi bi-check-circle-fill text-green me-2"></i>
                                Admin controls
                            </li>
                            <li class="mb-2">
                                <i class="bi bi-check-circle-fill text-green me-2"></i>
                                Custom integrations
                            </li>
                        </ul>

                        <button class="btn btn-outline-light w-100" disabled>Contact Sales</button>
                    </div>
                </div>
            </div>
        </div>

        <!-- Passion Project Message -->
        <div class="row justify-content-center mb-5">
            <div class="col-lg-8">
                <div class="card bg-dark border-0 shadow-lg p-4 text-center">
                    <div class="d-flex align-items-center justify-content-center mb-3">
                        <span class="fs-1">🌱</span>
                    </div>
                    <h3 class="fw-bold text-white mb-3">A Passion Project</h3>
                    <p class="text-white-50 mb-4">
                        Vynn is currently a passion project in development. While we're working on implementing 
                        our payment system, all features are available for testing. If you need additional AI credits 
                        or have any questions, please reach out through our <a href="/contact" class="text-green">Contact Us</a> page.
                    </p>
                    <p class="text-white-50 mb-0">
                        We appreciate your interest and support in making Vynn better!
                    </p>
                </div>
            </div>
        </div>

        <!-- Call to Action -->
        <div class="text-center py-5">
            <h2 class="fw-bold mb-4">Ready to Transform Your Writing?</h2>
            <p class="text-white-50 mb-4">
                Join thousands of writers who have already discovered the power of Vynn Editor.<br>
                Start your 14-day free trial today.
            </p>
            <div class="d-flex justify-content-center gap-3">
                <a href="/login" class="btn btn-green px-4 py-2">Start Free Trial</a>
                <a href="/tutorial" class="btn btn-outline-light px-4 py-2">Watch Demo</a>
            </div>
        </div>
    </div>
    <Footer />
</div>