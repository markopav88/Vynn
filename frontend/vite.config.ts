import { sveltekit } from '@sveltejs/kit/vite';
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

import { defineConfig } from 'vite';
import dotenv from 'dotenv';

// Load environment variables from .env file
dotenv.config({ path: '../backend/.env' });

/** @type {import('@sveltejs/kit').Config} */
const config = {
	plugins: [sveltekit()],
	// Define global constants - effectively replaces process.env.API_BASE_URL
	define: {
		'process.env.API_BASE_URL': JSON.stringify(process.env.API_BASE_URL)
	},
	css: {
		devSourcemap: false
	},
	preprocess: vitePreprocess(),

	kit: {
		adapter: adapter({
			// Set output to match Render's expected path
			pages: '.svelte-kit/output/client',
			assets: '.svelte-kit/output/client',
			fallback: 'index.html',
			precompress: false,
			strict: true
		})
	}
};

export default defineConfig(config);
