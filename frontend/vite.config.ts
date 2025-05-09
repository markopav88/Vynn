import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import dotenv from 'dotenv';

// Load environment variables from .env file
dotenv.config({ path: '../backend/.env' });

export default defineConfig({
	plugins: [sveltekit()],
	// Define global constants - effectively replaces process.env.API_BASE_URL
	define: {
		'process.env.API_BASE_URL': JSON.stringify(process.env.API_BASE_URL)
	},
	css: {
		devSourcemap: false
	}
});
