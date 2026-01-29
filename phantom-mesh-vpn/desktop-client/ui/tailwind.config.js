/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        phantom: {
          50: "#f0f5ff",
          100: "#e0eaff",
          200: "#c7d7ff",
          300: "#a5bfff",
          400: "#7a9cff",
          500: "#5575ff",
          600: "#3b4fff",
          700: "#2d39eb",
          800: "#2631c0",
          900: "#252f96",
          950: "#1a1f58",
        },
        mesh: {
          dark: "#0a0a0f",
          darker: "#050508",
          light: "#1a1a2e",
          accent: "#6366f1",
          success: "#10b981",
          warning: "#f59e0b",
          danger: "#ef4444",
        },
      },
      animation: {
        "pulse-slow": "pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite",
        "spin-slow": "spin 3s linear infinite",
        gradient: "gradient 8s ease infinite",
        glow: "glow 2s ease-in-out infinite alternate",
      },
      keyframes: {
        gradient: {
          "0%, 100%": { backgroundPosition: "0% 50%" },
          "50%": { backgroundPosition: "100% 50%" },
        },
        glow: {
          "0%": {
            boxShadow:
              "0 0 5px rgba(99, 102, 241, 0.2), 0 0 20px rgba(99, 102, 241, 0.1)",
          },
          "100%": {
            boxShadow:
              "0 0 20px rgba(99, 102, 241, 0.4), 0 0 40px rgba(99, 102, 241, 0.2)",
          },
        },
      },
      backgroundSize: {
        "300%": "300% 300%",
      },
    },
  },
  plugins: [],
};
