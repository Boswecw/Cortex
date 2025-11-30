/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {
      colors: {
        'cortex-black': '#0A0A0C',
        'cortex-deep': '#0E0F12',
        'slate-byte': '#15161A',
        'neural-gold': '#C9A46C',
        'ember-gold': '#F3C87D',
        'silver-neural': '#CCCCD6',
      },
      fontFamily: {
        sans: ['-apple-system', 'BlinkMacSystemFont', '"Segoe UI"', 'Roboto', 'sans-serif'],
      },
    },
  },
  plugins: [],
};
