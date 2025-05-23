<!-- frontend/src/routes/+page.svelte -->

<!--
    This is our root main page. It's the first page that loads when we go to the root URL.
    Svelte is a framework that allows us to build web applications using HTML, CSS, and Typescript.
    In a Svelte file we can have HTML, CSS, and Typescript code.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { logout, check_auth } from '$lib/ts/user';
	import logo from '$lib/assets/logo.png';
	import Navbar from '$lib/components/Navbar.svelte';
	import Footer from '$lib/components/Footer.svelte';
	import heroBackground from '$lib/assets/hero-background.png';
	import '$lib/assets/style/homepage.css';

	let isLoggedIn = false;
	let pageLoaded = false;
	let isTerminalVisible = true;
	let isTerminalMinimized = false;
	let hasLoadedOnce = false;

	function handleClose() {
		isTerminalVisible = false;
		isTerminalMinimized = false; // Reset minimized state when closing
		// Reappear after 2 seconds
		setTimeout(() => {
			isTerminalVisible = true;
		}, 50);
	}

	function handleMinimize() {
		isTerminalMinimized = !isTerminalMinimized;
		if (isTerminalMinimized) {
			isTerminalVisible = true; // Ensure terminal stays visible when minimized
		}
	}

	function handleMaximize() {
		window.location.href = '/tutorial';
	}

	onMount(() => {
		(async () => {
			document.title = "Vynn - AI Powered";
			try {
				const authCheckPromise = check_auth();
				const timeoutPromise = new Promise((resolve) => setTimeout(() => resolve(false), 3000));
				const isAuthenticated = await Promise.race([authCheckPromise, timeoutPromise]);

				if (isAuthenticated) {
					console.log('User is authenticated, redirecting to /drive');
					window.location.href = '/drive';
					return;
				}
			} catch (error) {
				console.error('Error checking authentication:', error);
			}

			pageLoaded = true;
		})();

		// Animation setup
		const handleScroll = () => {
			if (!hasLoadedOnce) {
				const fadeElements = document.querySelectorAll('.fade-in:not(.visible), .hero-fade:not(.visible)');
				fadeElements.forEach((el) => {
					const rect = el.getBoundingClientRect();
					if (rect.top < window.innerHeight * 0.8) {
						el.classList.add('visible');
						// If this is a hero section element, mark it as loaded
						if (el.classList.contains('hero-fade')) {
							hasLoadedOnce = true;
						}
					}
				});
			} else {
				// Only handle non-hero fade elements after initial load
				const fadeElements = document.querySelectorAll('.fade-in:not(.visible)');
				fadeElements.forEach((el) => {
					const rect = el.getBoundingClientRect();
					if (rect.top < window.innerHeight * 0.8) {
						el.classList.add('visible');
					}
				});
			}
		};

		window.addEventListener('scroll', handleScroll, { passive: true });
		handleScroll();

		// Return cleanup function (non-async)
		return () => {
			window.removeEventListener('scroll', handleScroll);
		};
	});
</script>

<div class="bg-black min-vh-100">
	<Navbar {isLoggedIn} />

	<!-- Hero Section with Background Image -->
	<section class="position-relative">
		<!-- Background Image -->
		<div class="position-absolute top-0 start-0 w-100 h-100" style="overflow: hidden;">
			<img src={heroBackground} alt="Hero Background" class="w-100 h-100 object-fit-cover opacity-25" />
		</div>

		<div class="container py-5">
			<!-- Header that spans full width -->
			<div class="row mb-4">
				<div class="col-12 text-center hero-fade hero-title">
					<h1 class="fw-bold" style="font-size: 6rem; line-height: 1.1;">
						Write using the Power of<br />
						<span class="text-green">Neovim + AI</span>
					</h1>
				</div>
			</div>

			<!-- Content in two columns -->
			<div class="row py-3 align-items-center">
				<!-- Left side: Text content -->
				<div class="col-md-{isTerminalVisible ? '6' : '12'} hero-fade hero-content text-center" 
					 class:visible={hasLoadedOnce}
					 style="margin-top: {isTerminalVisible ? '-2rem' : '0'}">
					<div class="content-wrapper" class:centered-content={!isTerminalVisible}>
						<p class="fs-4 text-white-50 mb-4">
							Unleash your creativity with the power of Neovim and AI assistance, helping writers craft stunning content with unmatched speed and efficiency.
						</p>

						<p class="fs-5 text-white-50 mb-5">
							Vynn combines the power of NeoVim's keyboard-centric editing with state-of-the-art AI to help you write faster,
							smarter, and more creatively. Whether you're drafting a novel, writing technical documentation, or crafting
							marketing copy, Vynn provides the tools you need to excel.
						</p>

						<div class="d-flex gap-4 hero-fade hero-buttons justify-content-center" class:visible={hasLoadedOnce}>
							<a href="/login" class="btn btn-green btn-lg px-4 py-2"> Try Vynn Editor </a>
							<a href="/tutorial" class="btn btn-outline-light btn-lg px-4 py-2"> View Tutorial </a>
						</div>
					</div>
				</div>

				<!-- Right side: Terminal -->
				{#if isTerminalVisible}
				<div class="col-md-6 mt-15 mt-md-0 d-flex justify-content-end hero-fade hero-terminal" 
					 class:visible={hasLoadedOnce}
					 style="margin-top: 0">
					<div class="position-relative ms-md-5 video-container" 
						 class:minimized={isTerminalMinimized}>
						<div class="terminal-window">
							<div class="terminal-header">
								<div class="terminal-buttons">
									<button type="button" class="terminal-button close" 
											on:click={() => isTerminalVisible = false}
											on:keydown={(e) => e.key === 'Enter' && (isTerminalVisible = false)}
											aria-label="Close terminal"></button>
									<button type="button" class="terminal-button minimize" 
											on:click={handleMinimize}
											on:keydown={(e) => e.key === 'Enter' && handleMinimize()}
											aria-label="Minimize terminal"></button>
									<button type="button" class="terminal-button maximize" 
											on:click={handleMaximize}
											on:keydown={(e) => e.key === 'Enter' && handleMaximize()}
											aria-label="Maximize terminal"></button>
								</div>
								<div class="terminal-title"></div>
							</div>
							<div class="video-wrapper">
								<video
									class="demo-video"
									autoplay
									loop
									muted
									playsinline
								>
									<source src="/video.mp4" type="video/mp4" />
									Your browser does not support the video tag.
								</video>
							</div>
						</div>
						<div class="glow-effect"></div>
					</div>
				</div>
				{/if}
			</div>
		</div>
	</section>

	<!-- Testimonials Section -->
	<section class="py-5 bg-black">
		<div class="container">
			<h2 class="text-center fw-bold mb-5 fade-in">Loved by Writers Worldwide</h2>

			<div class="row g-4">
				<!-- Testimonial 1 -->
				<div class="col-md-4 fade-in">
					<div
						class="p-4 rounded-3 h-100"
						style="background-color: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1);"
					>
						<div class="d-flex align-items-center mb-3">
							<div class="rounded-circle overflow-hidden me-3" style="width: 50px; height: 50px;">
								<img src="https://randomuser.me/api/portraits/women/44.jpg" alt="Sarah Davis" class="img-fluid" />
							</div>
							<div>
								<h5 class="mb-0 fw-bold">Sarah Davis</h5>
								<p class="text-white-50 small mb-0">Fiction Writer</p>
							</div>
						</div>
						<p class="text-white-50 mb-0">
							"Vynn has completely transformed how I write! The blend of Neovim's efficiency and AI assistance is
							exactly what I needed to boost my productivity."
						</p>
					</div>
				</div>

				<!-- Testimonial 2 -->
				<div class="col-md-4 fade-in">
					<div
						class="p-4 rounded-3 h-100"
						style="background-color: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1);"
					>
						<div class="d-flex align-items-center mb-3">
							<div class="rounded-circle overflow-hidden me-3" style="width: 50px; height: 50px;">
								<img src="https://randomuser.me/api/portraits/men/32.jpg" alt="Mark Thompson" class="img-fluid" />
							</div>
							<div>
								<h5 class="mb-0 fw-bold">Mark Thompson</h5>
								<p class="text-white-50 small mb-0">Journalist</p>
							</div>
						</div>
						<p class="text-white-50 mb-0">
							"It's a life-saver. Writing for a news outlet requires speed and accuracy - Vynn delivers both. I can
							research, draft, and edit articles faster than I thought possible."
						</p>
					</div>
				</div>

				<!-- Testimonial 3 -->
				<div class="col-md-4 fade-in">
					<div
						class="p-4 rounded-3 h-100"
						style="background-color: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1);"
					>
						<div class="d-flex align-items-center mb-3">
							<div class="rounded-circle overflow-hidden me-3" style="width: 50px; height: 50px;">
								<img src="https://randomuser.me/api/portraits/women/68.jpg" alt="Emily Rodriguez" class="img-fluid" />
							</div>
							<div>
								<h5 class="mb-0 fw-bold">Emily Rodriguez</h5>
								<p class="text-white-50 small mb-0">Content Creator</p>
							</div>
						</div>
						<p class="text-white-50 mb-0">
							"The AI assistance in Vynn feels like having a writing partner who understands your style. It helps me
							overcome writer's block and enhances my creativity."
						</p>
					</div>
				</div>
			</div>
		</div>
	</section>

	<!-- Pricing Section -->
	<section class="py-5 bg-black">
		<div class="container">
			<h2 class="text-center fw-bold mb-3 fade-in">Simple, Transparent Pricing</h2>
			<p class="text-center text-white-50 mb-5 fade-in">Choose the plan that works best for you</p>

			<div class="row g-4 justify-content-center mt-5 pt-4">
				<!-- Starter Plan -->
				<div class="col-md-4 fade-in">
					<div class="card h-100 bg-dark text-white border-0">
						<div class="card-body p-4">
							<h3 class="card-title fw-bold mb-4">Starter</h3>
							<p class="text-white-50 mb-4">Perfect for individual writers</p>

							<div class="mb-4">
								<h2 class="fw-bold mb-0">$9</h2>
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

							<a href="/signup" class="btn btn-outline-light w-100">Get Started</a>
						</div>
					</div>
				</div>

				<!-- Pro Plan -->
				<div class="col-md-4 fade-in">
					<div class="card h-100 bg-dark text-white border-0 position-relative featured-card">
						<div class="position-absolute top-0 start-50 translate-middle">
							<span class="badge bg-green px-4 py-2 rounded-pill mt-2 mb-5 popular-badge">Popular</span>
						</div>
						<div class="card-body p-4">
							<h3 class="card-title fw-bold mb-4">Pro</h3>
							<p class="text-white-50 mb-4">For professional writers</p>

							<div class="mb-4">
								<h2 class="fw-bold mb-0">$19</h2>
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

							<a href="/signup" class="btn btn-green w-100">Get Started</a>
						</div>
					</div>
				</div>

				<!-- Team Plan -->
				<div class="col-md-4 fade-in">
					<div class="card h-100 bg-dark text-white border-0">
						<div class="card-body p-4">
							<h3 class="card-title fw-bold mb-4">Team</h3>
							<p class="text-white-50 mb-4">For writing teams</p>

							<div class="mb-4">
								<h2 class="fw-bold mb-0">$49</h2>
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

							<a href="/contact" class="btn btn-outline-light w-100">Contact Sales</a>
						</div>
					</div>
				</div>
			</div>
		</div>
	</section>

	<!-- Call to Action Section -->
	<section class="py-5 bg-black">
		<div class="container">
			<div
				class="p-5 rounded-4 text-center fade-in"
				style="background: linear-gradient(rgba(16,185,129,0.1), rgba(0,0,0,0.5));"
			>
				<h2 class="fw-bold mb-4">Ready to Transform Your Writing?</h2>
				<p class="text-white-50 mb-4 mx-auto" style="max-width: 600px;">
					Join thousands of writers who have already discovered the power of Vynn Editor. Start your 14-day free trial
					today.
				</p>
				<div class="d-flex justify-content-center gap-3">
					<a href="/signup" class="btn btn-green btn-lg px-4">Start Free Trial</a>
					<a href="/demo" class="btn btn-outline-light btn-lg px-4">Watch Demo</a>
				</div>
			</div>
		</div>
	</section>

	<!-- Use the Footer component -->
	<Footer />
</div>