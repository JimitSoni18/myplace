import { defineConfig, type Plugin } from "vite";
import solidPlugin from "vite-plugin-solid";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

const target = process.env.APP_TARGET || "admin";

function htmlEntryPlugin(entryHtml: string): Plugin {
  return {
    name: "html-entry-rewriter",
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        const url = req.url || "/";
        // Prevent cross-app access: owner server should not serve admin, and vice versa
        if (target === "owner" && (url.startsWith("/admin.html") || url === "/admin" || url.startsWith("/admin?"))) {
          res.statusCode = 404;
          res.end("Not Found");
          return;
        }
        if (target === "admin" && (url.startsWith("/owner.html") || url === "/owner" || url.startsWith("/owner?"))) {
          res.statusCode = 404;
          res.end("Not Found");
          return;
        }
        if (
          url === "/" ||
          url === "/index.html" ||
          url.startsWith("/?") ||
          (!url.startsWith("/@") &&
            !url.startsWith("/src") &&
            !url.startsWith("/node_modules") &&
            !url.startsWith("/api") &&
            !url.startsWith("/static") &&
            !url.includes("."))
        ) {
          req.url = `/${entryHtml}` + (url.includes("?") ? url.slice(url.indexOf("?")) : "");
        }
        next();
      });
    },
  };
}

export default defineConfig({
  plugins: [tailwindcss(), solidPlugin(), htmlEntryPlugin(`${target}.html`)],
  define: {
    __APP_TARGET__: JSON.stringify(target),
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    port: target === "owner" ? 3002 : 3001,
    proxy: {
      "/api": {
        target: process.env.VITE_BACKEND_URL || "http://127.0.0.1:8080",
        changeOrigin: true,
      },
      "/static": {
        target: process.env.VITE_BACKEND_URL || "http://127.0.0.1:8080",
        changeOrigin: true,
      },
    },
  },
  build: {
    target: "esnext",
    outDir: `dist/${target}`,
    emptyOutDir: true,
    rollupOptions: {
      input: {
        index: path.resolve(__dirname, `${target}.html`),
      },
    },
  },
});

