import postcssImport from 'postcss-import';
import tailwindcssNesting from 'tailwindcss/nesting/index.js';
import postcssNesting from 'postcss-nesting';
import autoprefixer from 'autoprefixer';
import tailwindcss from 'tailwindcss';

export default {
  plugins: [
    postcssImport,
    tailwindcssNesting(postcssNesting),
    autoprefixer,
    tailwindcss,
  ]
}