<script lang="ts">
	import logo from '$lib/assets/logo.png';
</script>

<footer class="py-5 bg-black">
	<div class="container">
		<div class="row g-4">
			<div class="col-lg-3 col-md-6">
				<h5 class="fw-bold mb-3">Product</h5>
				<ul class="list-unstyled">
					<li class="mb-2"><a href="/features" class="text-white-50 text-decoration-none">Features</a></li>
					<li class="mb-2"><a href="/pricing" class="text-white-50 text-decoration-none">Pricing</a></li>
					<li class="mb-2"><a href="/security" class="text-white-50 text-decoration-none">Security</a></li>
					<li class="mb-2"><a href="/roadmap" class="text-white-50 text-decoration-none">Roadmap</a></li>
				</ul>
			</div>

			<div class="col-lg-3 col-md-6">
				<h5 class="fw-bold mb-3">Resources</h5>
				<ul class="list-unstyled">
					<li class="mb-2"><a href="/documentation" class="text-white-50 text-decoration-none">Documentation</a></li>
					<li class="mb-2"><a href="/tutorials" class="text-white-50 text-decoration-none">Tutorials</a></li>
					<li class="mb-2"><a href="/blog" class="text-white-50 text-decoration-none">Blog</a></li>
					<li class="mb-2"><a href="/community" class="text-white-50 text-decoration-none">Community</a></li>
				</ul>
			</div>

			<div class="col-lg-3 col-md-6">
				<h5 class="fw-bold mb-3">Company</h5>
				<ul class="list-unstyled">
					<li class="mb-2"><a href="/about" class="text-white-50 text-decoration-none">About</a></li>
					<li class="mb-2"><a href="/careers" class="text-white-50 text-decoration-none">Careers</a></li>
					<li class="mb-2"><a href="/contact" class="text-white-50 text-decoration-none">Contact</a></li>
					<li class="mb-2"><a href="/partners" class="text-white-50 text-decoration-none">Partners</a></li>
				</ul>
			</div>

			<div class="col-lg-3 col-md-6">
				<h5 class="fw-bold mb-3">Legal</h5>
				<ul class="list-unstyled">
					<li class="mb-2"><a href="/privacy" class="text-white-50 text-decoration-none">Privacy</a></li>
					<li class="mb-2"><a href="/terms" class="text-white-50 text-decoration-none">Terms</a></li>
					<li class="mb-2"><a href="/license" class="text-white-50 text-decoration-none">License</a></li>
				</ul>
			</div>
		</div>

		<div class="d-flex justify-content-between align-items-center mt-5 pt-4">
			<div class="d-flex align-items-center">
				<a href="/" class="text-decoration-none">
					<img src={logo} alt="Vynn Logo" height="30" width="30" class="me-2" />
					<span class="text-white fw-bold">Vynn</span>
				</a>
			</div>

			<div class="d-flex gap-3">
				<a href="https://twitter.com" class="text-white-50" aria-label="Twitter">
					<i class="bi bi-twitter"></i>
				</a>
				<a href="https://github.com" class="text-white-50" aria-label="GitHub">
					<i class="bi bi-github"></i>
				</a>
				<a href="https://linkedin.com" class="text-white-50" aria-label="LinkedIn">
					<i class="bi bi-linkedin"></i>
				</a>
			</div>
		</div>
	</div>
</footer>
