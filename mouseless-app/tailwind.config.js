/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./frontend/index.html",
    "./frontend/src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'overlay-bg': 'rgba(0, 0, 0, 0.1)',
        'grid-line': 'rgba(0, 122, 255, 0.6)',
        'grid-label': 'rgba(0, 122, 255, 0.9)',
        'area-border': 'rgba(255, 255, 255, 0.3)',
        'prediction-high': 'rgba(0, 255, 0, 0.8)',
        'prediction-medium': 'rgba(255, 255, 0, 0.8)',
        'prediction-low': 'rgba(255, 0, 0, 0.8)',
      },
      animation: {
        'fade-in': 'fadeIn 0.3s ease-in-out',
        'scale-in': 'scaleIn 0.2s ease-out',
        'pulse-soft': 'pulseSoft 2s ease-in-out infinite',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        scaleIn: {
          '0%': { transform: 'scale(0.8)', opacity: '0' },
          '100%': { transform: 'scale(1)', opacity: '1' },
        },
        pulseSoft: {
          '0%, 100%': { opacity: '0.6' },
          '50%': { opacity: '1' },
        },
      },
    },
  },
  plugins: [],
}