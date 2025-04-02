<script lang="ts">
	import { onMount } from 'svelte';

	import { attempt_login, Login, check_auth } from '$lib/ts/user';
	import logo from '$lib/assets/logo.png';

	import Navbar from '$lib/components/Navbar.svelte';

	import '$lib/assets/style/login.css'

	let email = '';
	let password = '';
	let showPassword = false;
	let rememberMe = false;
	let isLoggedIn = false;
	let isLoading = false;
	let errorMessage = '';
	let pageLoaded = false;
	let showSuccessMessage = false;

	// Check for auth token and registered=true query parameter
	onMount(async () => {
		try {
			// Check authentication status using backend API with a timeout
			const authCheckPromise = check_auth();
			const timeoutPromise = new Promise((resolve) => setTimeout(() => resolve(false), 3000));

			// Race between auth check and timeout
			const isAuthenticated = await Promise.race([authCheckPromise, timeoutPromise]);

			if (isAuthenticated) {
				console.log('User is authenticated, redirecting to /drive');
				window.location.href = '/drive';
				return;
			}
		} catch (error) {
			console.error('Error checking authentication:', error);
			// Continue loading the page even if auth check fails
		}

		// Trigger fade-in animation after component mounts
		setTimeout(() => {
			pageLoaded = true;
		}, 100);

		// Check if user just registered
		const urlParams = new URLSearchParams(window.location.search);
		if (urlParams.get('registered') === 'true') {
			showSuccessMessage = true;
			// Auto-hide after 5 seconds
			setTimeout(() => {
				showSuccessMessage = false;
			}, 5000);
		}
	});

	// Toggle password visibility
	function togglePasswordVisibility() {
		showPassword = !showPassword;
	}

	async function handleSubmit() {
		try {
			isLoading = true;
			errorMessage = '';

			// Create Login object
			const loginPayload = new Login(email, password);

			// Call the attempt_login function
			const success = await attempt_login(loginPayload);

			// Check the boolean return value
			if (success) {
				// If successful, redirect to dashboard
				window.location.href = '/drive';
			} else {
				// If the function returns false but doesn't throw an error
				errorMessage = 'Invalid email or password. Please try again.';
			}
		} catch (error: any) {
			console.error('Login error:', error);
			errorMessage = error.message || 'Failed to login. Please try again.';
		} finally {
			isLoading = false;
		}
	}
</script>

<div class="bg-black min-vh-100 d-flex flex-column">
	<Navbar {isLoggedIn} />

	<!-- Success Message (above login UI) -->
	{#if showSuccessMessage}
		<div class="container mt-4">
			<div class="alert alert-success alert-dismissible fade show" role="alert">
				<strong>Success!</strong> Your account has been created successfully. Please sign in.
				<button
					type="button"
					class="btn-close"
					data-bs-dismiss="alert"
					aria-label="Close"
					on:click={() => (showSuccessMessage = false)}
				></button>
			</div>
		</div>
	{/if}

	<div class="container flex-grow-1 d-flex align-items-center justify-content-center py-5 my-5">
		<div
			class="card bg-dark text-white border-0 shadow-lg fade-in {pageLoaded ? 'visible' : ''}"
			style="max-width: 450px; width: 100%;"
		>
			<div class="card-body p-4 p-md-5">
				<!-- Logo and Title -->
				<div class="text-center mb-4 fade-element">
					<img src={logo} alt="Vynn Logo" height="50" width="50" class="mb-3" />
					<h2 class="fw-bold">Welcome back</h2>
					<p class="text-white-50">Sign in to your account</p>
				</div>

				<!-- Error Message -->
				{#if errorMessage}
					<div class="alert alert-danger mb-4" role="alert">
						{errorMessage}
					</div>
				{/if}

				<!-- Login Form -->
				<form on:submit|preventDefault={handleSubmit} class="fade-element">
					<!-- Email -->
					<div class="mb-4">
						<label for="email" class="form-label small text-white-50">Email</label>
						<input
							type="email"
							class="form-control bg-black text-white border-dark"
							id="email"
							bind:value={email}
							placeholder="your@example.com"
							required
						/>
					</div>

					<!-- Password -->
					<div class="mb-4">
						<div class="d-flex justify-content-between">
							<label for="password" class="form-label small text-white-50">Password</label>
							<a href="/forgot-password" class="text-green small">Forgot password?</a>
						</div>
						<div class="input-group">
							<input
								type={showPassword ? 'text' : 'password'}
								class="form-control bg-black text-white border-dark"
								id="password"
								bind:value={password}
								required
							/>
							<button
								type="button"
								class="input-group-text bg-black border-dark text-white-50"
								on:click={togglePasswordVisibility}
								aria-label="Toggle password visibility"
							>
								<i class="bi {showPassword ? 'bi-eye-slash' : 'bi-eye'}"></i>
							</button>
						</div>
					</div>

					<!-- Remember Me -->
					<div class="mb-4">
						<div class="form-check">
							<input class="form-check-input" type="checkbox" id="rememberMe" bind:checked={rememberMe} />
							<label class="form-check-label small text-white-50" for="rememberMe"> Remember me </label>
						</div>
					</div>

					<!-- Submit Button -->
					<button type="submit" class="btn btn-green w-100 py-2 mb-3" disabled={isLoading}>
						{#if isLoading}
							<span class="spinner-border spinner-border-sm me-2" role="status" aria-hidden="true"></span>
							Signing in...
						{:else}
							Sign in
						{/if}
					</button>

					<!-- Signup Link -->
					<div class="text-center">
						<span class="text-white-50 small">Don't have an account?</span>
						<a href="/signup" class="text-green small ms-1">Create account</a>
					</div>
				</form>

				<!-- Social Login -->
				<div class="mt-4 fade-element">
					<div class="d-flex align-items-center mb-3">
						<hr class="flex-grow-1 border-dark" />
						<span class="mx-3 text-white-50 small">Or continue with</span>
						<hr class="flex-grow-1 border-dark" />
					</div>

					<div class="d-flex justify-content-center gap-3">
						<button class="btn btn-link text-white-50 p-0 border-0" aria-label="GitHub">
							<i class="bi bi-github"></i>
						</button>
						<button class="btn btn-link text-white-50 p-0 border-0" aria-label="Google">
							<i class="bi bi-google"></i>
						</button>
						<button class="btn btn-link text-white-50 p-0 border-0" aria-label="Twitter">
							<i class="bi bi-twitter"></i>
						</button>
					</div>
				</div>
			</div>
		</div>
	</div>
</div>