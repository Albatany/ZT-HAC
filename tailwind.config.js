export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        bg: "#0b0d10", surface: "#12151a", raised: "#181c22", line: "#232932",
        ink: "#e6e9ee", mute: "#8a93a2", accent: "#2dd4bf", danger: "#f87171", warn: "#fbbf24",
      },
      fontFamily: {
        sans: ["Inter", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono", "ui-monospace", "monospace"],
      },
      keyframes: {
        rise: { "0%": { opacity: 0, transform: "translateY(10px)" }, "100%": { opacity: 1, transform: "none" } },
        toast: { "0%": { opacity: 0, transform: "translateX(24px)" }, "100%": { opacity: 1, transform: "none" } },
        beat: { "0%,100%": { opacity: 1, transform: "scale(1)" }, "50%": { opacity: 0.4, transform: "scale(1.6)" } },
        pop: { "0%": { opacity: 0, transform: "scale(.96)" }, "100%": { opacity: 1, transform: "none" } },
      },
      animation: {
        rise: "rise .35s ease-out both", toast: "toast .3s ease-out both",
        beat: "beat 1.8s ease-in-out infinite", pop: "pop .25s ease-out both",
      },
    },
  },
  plugins: [],
};
