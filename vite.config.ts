import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import templateCompilerOptions from "@tresjs/core/template-compiler-options";
// @ts-expect-error type error without @types/node package
import process from "node:process";
// @ts-expect-error type error without @types/node package
import { readFileSync, existsSync } from "node:fs";
// @ts-expect-error type error without @types/node package
import { dirname, resolve } from "node:path";
// @ts-expect-error type error without @types/node package
import { fileURLToPath } from "node:url";

const host = process.env.TAURI_DEV_HOST;
const MIN_DEV_PORT = 20000;
const DEFAULT_DEV_PORT = 20015;
const root_dir = dirname(fileURLToPath(import.meta.url));

function read_dev_port(): number {
  const from_env = Number(process.env.RGAIN_DEV_PORT);
  if (Number.isFinite(from_env) && from_env >= MIN_DEV_PORT && from_env <= 65535) {
    return Math.floor(from_env);
  }

  const cfg_path = resolve(root_dir, "rgain.dev.json");
  if (existsSync(cfg_path)) {
    try {
      const raw = JSON.parse(readFileSync(cfg_path, "utf8")) as { dev_port?: number };
      const port = Number(raw.dev_port);
      if (Number.isFinite(port) && port >= MIN_DEV_PORT && port <= 65535) {
        return Math.floor(port);
      }
    } catch {
      // fall through to default
    }
  }
  return DEFAULT_DEV_PORT;
}

const dev_port = read_dev_port();

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [
    vue({
      // TresJS custom elements (TresMesh, …)
      ...templateCompilerOptions,
    }),
  ],

  clearScreen: false,
  server: {
    port: dev_port,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: dev_port + 1,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
