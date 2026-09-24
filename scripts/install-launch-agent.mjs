#!/usr/bin/env node
// Keeps the desktop app running in the background so the terminal CLI always
// has a renderer to talk to.
//
//   node scripts/install-launch-agent.mjs [--app <path>]
//   node scripts/install-launch-agent.mjs --uninstall
//
// The app runs as a menu bar accessory (see src-tauri/Info.plist), so "running"
// means a tray icon and no window.

import { execFileSync } from "node:child_process";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";

const LABEL = "dev.yoophi.mermaid-live";
const DEFAULT_APP = "/Applications/Mermaid Live.app";

const args = process.argv.slice(2);
const uninstall = args.includes("--uninstall");
const appIndex = args.indexOf("--app");
const appPath = appIndex === -1 ? DEFAULT_APP : args[appIndex + 1];

const agentsDir = path.join(os.homedir(), "Library", "LaunchAgents");
const plistPath = path.join(agentsDir, `${LABEL}.plist`);
const domain = `gui/${process.getuid()}`;

function run(command, commandArgs) {
  try {
    execFileSync(command, commandArgs, { stdio: ["ignore", "pipe", "pipe"] });
    return true;
  } catch {
    return false;
  }
}

/** Removing first makes the script safe to run twice. */
function unload() {
  if (!run("launchctl", ["bootout", `${domain}/${LABEL}`])) {
    run("launchctl", ["unload", "-w", plistPath]);
  }
}

if (uninstall) {
  unload();
  rmSync(plistPath, { force: true });
  console.log(`[launch-agent] removed ${plistPath}`);
  process.exit(0);
}

const executable = path.join(appPath, "Contents", "MacOS", "mermaid-live");

const plist = `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
\t<key>Label</key>
\t<string>${LABEL}</string>
\t<key>ProgramArguments</key>
\t<array>
\t\t<string>${executable}</string>
\t</array>
\t<key>RunAtLoad</key>
\t<true/>
\t<!-- Restart after a crash, but respect Quit from the tray menu. -->
\t<key>KeepAlive</key>
\t<dict>
\t\t<key>SuccessfulExit</key>
\t\t<false/>
\t</dict>
\t<key>ProcessType</key>
\t<string>Interactive</string>
</dict>
</plist>
`;

mkdirSync(agentsDir, { recursive: true });
unload();
writeFileSync(plistPath, plist);

if (!run("launchctl", ["bootstrap", domain, plistPath])) {
  run("launchctl", ["load", "-w", plistPath]);
}

console.log(`[launch-agent] installed ${plistPath}`);
console.log(`[launch-agent] launching ${executable}`);
console.log("[launch-agent] remove it with: node scripts/install-launch-agent.mjs --uninstall");
