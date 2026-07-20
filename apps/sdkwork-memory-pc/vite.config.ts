import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [tailwindcss(), react()],
  resolve: {
    dedupe: ["react", "react-dom", "react-router", "react-router-dom"],
  },
  server: {
    host: "127.0.0.1",
    port: 3910,
  },
  preview: {
    host: "127.0.0.1",
    port: 4910,
  },
  build: {
    sourcemap: true,
    target: "es2022",
  },
});
