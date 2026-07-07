import { rm } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const dryRun = process.argv.includes("--dry-run");

const targets = [
  "node_modules",
  "apps/desktop/node_modules",
  "apps/desktop/dist",
  "apps/desktop/src-tauri/target",
  "apps/desktop/src-tauri/gen",
  "dist",
  "dist-ssr",
  "build",
  "coverage",
];

async function removeTarget(relativePath) {
  const absolutePath = path.join(rootDir, relativePath);

  if (dryRun) {
    console.log(`[clean] would remove ${relativePath}`);
    return;
  }

  await rm(absolutePath, { force: true, recursive: true });
  console.log(`[clean] removed ${relativePath}`);
}

for (const target of targets) {
  await removeTarget(target);
}
