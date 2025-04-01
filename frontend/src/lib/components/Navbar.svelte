<script lang="ts">
	import { onMount } from 'svelte';
	import { logout } from '$lib/ts/user';
	import logo from '$lib/assets/logo.png';

	export let isLoggedIn = false;

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
</script>

<nav class="navbar navbar-expand navbar-dark bg-black">
	<div class="container-fluid">
		<!-- Logo and Brand Name -->
		<a class="navbar-brand d-flex align-items-center" href="/">
			<img src={logo} alt="Vynn Logo" class="me-2" height="60" width="60" />
			<span class="text-white fw-semibold">Vynn</span>
		</a>

		<!-- Navigation Links - Always Centered -->
		<div class="navbar-collapse justify-content-center flex-grow-1 mr-5">
			<ul class="navbar-nav mx-auto text-center">
				<li class="nav-item mx-3">
					<a class="nav-link" href="/editor">Editor</a>
				</li>
				<li class="nav-item mx-3">
					<a class="nav-link" href="/tools">AI Tools</a>
				</li>
				<li class="nav-item mx-3">
					<a class="nav-link" href="/pricing">Pricing</a>
				</li>
				<li class="nav-item mx-3">
					<a class="nav-link" href="/community">Community</a>
				</li>
			</ul>
		</div>

		<!-- Action Buttons -->
		<div>
			{#if isLoggedIn}
				<button on:click={handleLogout} class="btn btn-danger btn-sm rounded-pill"> Logout </button>
			{:else}
				<a href="/signup" class="btn btn-green btn-sm rounded-pill"> Get Started </a>
			{/if}
		</div>
	</div>
</nav>

<style>
	nav {
		border-bottom: none;
	}

	.navbar-brand span {
		letter-spacing: 0.05em;
		font-size: 1.5rem;
	}

	/* Ensure navbar stays horizontal on all screen sizes */
	@media (max-width: 992px) {
		.navbar-collapse {
			display: flex !important;
		}

		.navbar-nav {
			flex-direction: row !important;
		}

		.nav-item {
			white-space: nowrap;
		}
	}

	/* Adjust spacing on very small screens */
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

	/* Add these styles to your Navbar component */
	.nav-link {
		color: rgba(255, 255, 255, 0.8) !important;
		transition: color 0.2s ease;
		cursor: pointer;
	}

	.nav-link:hover {
		color: var(--color-primary) !important;
	}

	/* Prevent any hover-based navigation */
	a {
		pointer-events: auto;
	}

	a:hover {
		text-decoration: none;
	}
</style>
