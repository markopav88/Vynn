<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { logout, get_current_user, get_profile_image_url, check_auth } from '$lib/ts/user';
	import logo from '$lib/assets/logo.png';
	import profileDefault from '$lib/assets/profile-image.png';

	export let isLoggedIn = false;
	let userId: number | null = null;
	let profileImage = profileDefault;

	onMount(async () => {
		if (isLoggedIn) {
			try {
				// Get current user data
				const user = await get_current_user();
				if (user && user.id) {
					userId = user.id;

					// Try to load profile image with timestamp to prevent caching
					const timestamp = new Date().getTime();
					const imageUrl = `${get_profile_image_url(user.id)}?t=${timestamp}`;

					// Check if the image exists
					const response = await fetch(imageUrl, { method: 'HEAD' });
					if (response.ok) {
						profileImage = imageUrl;
						console.log('got image: ', imageUrl)
					}
				}
			} catch (error) {
				console.error('Error loading user profile:', error);
			}
		}
	});

	async function handleLogout() {
		try {
			console.log('Attempting to logout...');
			const success = await logout();

			if (success) {
				console.log('Logout successful, redirecting to homepage');
				window.location.href = '/';
			} else {
				console.error('Logout failed');
				alert('Failed to logout. Please try again.');
			}
		} catch (error) {
			console.error('Error during logout:', error);
		}
	}

	// Add handler for account page navigation
	function goToAccount() {
		window.location.href = '/account';
	}

	// Function to handle logo click
	async function handleLogoClick() {
		const isAuthenticated = await check_auth();
		if (isAuthenticated) {
			goto('/drive'); // Redirect to /drive if authenticated
		} else {
			goto('/'); // Redirect to home if not authenticated
		}
	}

	// Function to handle My Drive click
	async function handleDriveClick() {
		const isAuthenticated = await check_auth();
		if (isAuthenticated) {
			goto('/drive'); // Redirect to /drive if authenticated
		} else {
			goto('/login'); // Redirect to /login if not authenticated
		}
	}
</script>

<nav class="navbar navbar-expand navbar-dark bg-black">
	<div class="container-fluid">
		<!-- Logo and Brand Name -->
		<button class="navbar-brand d-flex align-items-center" type="button" tabindex="0" on:click={handleLogoClick} style="border: none; background: none; padding: 0;">
			<img src={logo} alt="Vynn Logo" class="me-2" height="60" width="60" />
			<span class="text-white fw-semibold">Vynn</span>
		</button>

		<!-- Navigation Links - Always Centered -->
		<div class="navbar-collapse justify-content-center flex-grow-1 mr-5">
			<ul class="navbar-nav mx-auto text-center">
				<li class="nav-item mx-3">
					<a class="nav-link" href="/drive" on:click|preventDefault={handleDriveClick}>
						My Drive
					</a>
				</li>
				<li class="nav-item mx-3">
					<a class="nav-link" href="/tutorial">Tutorial</a>
				</li>
				<li class="nav-item mx-3">
					<a class="nav-link" href="/plans">Pro Plans</a>
				</li>
				<li class="nav-item mx-3">
					<a class="nav-link" href="/help">Help</a>
				</li>
			</ul>
		</div>

		<!-- Action Buttons -->
		<div>
			{#if isLoggedIn}
				<div class="dropdown">
					<button
						class="btn p-0 border-0 bg-transparent"
						data-bs-toggle="dropdown"
						aria-expanded="false"
						aria-haspopup="true"
						aria-label="Profile menu"
					>
						<img
							src={profileImage}
							alt="Profile"
							class="rounded-circle profile-img"
							style="width: 40px; height: 40px; border: 2px solid var(--color-primary); object-fit: cover;"
							on:error={() => (profileImage = profileDefault)}
						/>
					</button>
					<ul class="dropdown-menu dropdown-menu-end dropdown-menu-dark">
						<li>
							<button class="dropdown-item" on:click={goToAccount}>
								<i class="bi bi-person me-2"></i> My Account
							</button>
						</li>
						<li><hr class="dropdown-divider" /></li>
						<li>
							<button class="dropdown-item text-danger" on:click={handleLogout}>
								<i class="bi bi-box-arrow-right me-2"></i> Sign Out
							</button>
						</li>
					</ul>
				</div>
			{:else}
				<a href="/signup" class="btn btn-green btn-sm rounded-pill"> Get Started </a>
			{/if}
		</div>
	</div>
</nav>

<style>
	nav {
		border-bottom: none;
		position: relative;
		isolation: isolate;
		z-index: 1000;
		height: 60px;
		min-height: 60px;
		max-height: 60px;
		padding-top: 0;
		padding-bottom: 0;
		display: flex;
		align-items: center;
		box-sizing: border-box;
		background-color: rgba(10, 23, 33, 0.95);
		width: 100%;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
	}

	.navbar-brand span {
		letter-spacing: 0.05em;
		font-size: 1.5rem;
	}

	/* Ensure navbar stays horizontal on all screen sizes */
	@media (max-width: 992px) {
		.navbar-collapse {
			display: flex;
		}

		.navbar-nav {
			flex-direction: row;
		}

		.nav-item {
			white-space: nowrap;
		}
	}

	@media (max-width: 576px) {
		.nav-item {
			margin-left: 0.5rem !important;
			margin-right: 0.5rem !important;
		}

		.navbar-brand img {
			height: 40px;
			width: 40px;
		}

		.navbar-brand span {
			font-size: 1.2rem;
		}
	}

	/* Profile image hover effect */
	.profile-img {
		transition: all 0.2s ease;
	}

	.profile-img:hover {
		transform: scale(1.05);
		box-shadow: 0 0 10px rgba(16, 185, 129, 0.5);
	}

</style>
