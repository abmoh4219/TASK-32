/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./frontend/src/**/*.rs",
    "./index.html",
  ],
  theme: {
    extend: {
      colors: {
        "bg-primary":   "#0A0A0F",
        "bg-secondary": "#12121A",
        "bg-card":      "#1A1A28",
        "bg-hover":     "#1F1F32",
        "gold-400":     "#F5C518",
        "gold-500":     "#E8A900",
        "gold-600":     "#CC8800",
        "text-primary":   "#F0F0F5",
        "text-secondary": "#A0A0B0",
        "text-gold":      "#F5C518",
      },
      backgroundImage: {
        "gradient-gold": "linear-gradient(135deg, #F5C518 0%, #E8A900 50%, #CC8800 100%)",
      },
      boxShadow: {
        "gold": "0 0 20px rgba(245,197,24,0.15)",
        "gold-strong": "0 0 30px rgba(245,197,24,0.30)",
      },
      keyframes: {
        shimmer: {
          "0%": { backgroundPosition: "-1000px 0" },
          "100%": { backgroundPosition: "1000px 0" },
        },
      },
      animation: {
        shimmer: "shimmer 2s infinite linear",
      },
    },
  },
  plugins: [],
};
