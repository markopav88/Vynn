/** @type {import('tailwindcss').Config} */
export default {
  content: {
    files: ['./src/**/*.{html,js,svelte,ts}'],
    transform: {
      svelte: (content) => {
        const contentWithoutStyleBlocks = content.replace(/<style[^]+?<\/style>/gi, '')
        return contentWithoutStyleBlocks.match(/[A-Za-z0-9-_/:]*[A-Za-z0-9-_/]+/g) || []
      }
    }
  },
  theme: {
    extend: {},
  },
  plugins: [
    require('@tailwindcss/typography')
  ]
} 