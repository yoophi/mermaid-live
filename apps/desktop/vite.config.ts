import path from "node:path";
import { execFileSync } from "node:child_process";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import packageJson from "./package.json";

function gitOutput(args: string[]) {
  try {
    return execFileSync("git", args, {
      cwd: path.resolve(__dirname, "../.."),
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    }).trim();
  } catch {
    return undefined;
  }
}

function buildVersion() {
  const baseVersion = packageJson.version;
  const commitHash = (process.env.GITHUB_SHA ?? process.env.COMMIT_SHA ?? gitOutput(["rev-parse", "HEAD"]))
    ?.trim()
    .slice(0, 7);

  if (!commitHash) {
    return `${baseVersion}-unknown`;
  }

  const status = gitOutput(["status", "--porcelain"]);
  const dirty = status !== undefined && status.length > 0;
  const tag =
    process.env.GITHUB_REF_TYPE === "tag"
      ? process.env.GITHUB_REF_NAME?.trim()
      : gitOutput(["tag", "--points-at", "HEAD"])
          ?.split("\n")
          .map((value) => value.trim())
          .find((value) => value === baseVersion);

  if (!dirty && tag === baseVersion) {
    return baseVersion;
  }

  return `${baseVersion}-${commitHash}${dirty ? "-dirty" : ""}`;
}

export default defineConfig({
  plugins: [react(), tailwindcss()],
  define: {
    __MERMAID_LIVE_BUILD_VERSION__: JSON.stringify(buildVersion()),
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
});
