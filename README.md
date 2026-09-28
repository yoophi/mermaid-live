# Mermaid Live

A Tauri desktop app for editing Mermaid diagrams with a live preview, and for rendering Mermaid charts inline in the terminal.

## Features

- **Editor with live preview** - edit Mermaid source and see the chart update as you type, with zoom-to-fit, multiple windows and tabs, and `Cmd+S` to save.
- **Menu bar render service** - the app runs as a menu bar accessory with no window and no Dock icon. It serves render requests over a local socket and rasterises charts in a hidden webview.
- **Clipboard inbox** - copied Mermaid charts are detected and staged in the tray menu, with a notification, so they can be opened in a window later.
- **Terminal inline rendering** - `mmdcat` draws charts in the terminal with the kitty graphics protocol, using the same Mermaid version and theme as the desktop preview.

## Stack

- pnpm monorepo, plus a Cargo workspace under `crates/`
- Tauri
- React + Vite + TypeScript
- shadcn/ui-style shared components
- Feature-Sliced Design frontend
- Hexagonal architecture native layer

## Repository layout

| Path | Contents |
|---|---|
| `apps/desktop` | Tauri desktop app (React frontend in `src`, native side in `src-tauri`) |
| `crates/mermaid-core` | Chart detection, raster size contract and render socket protocol, shared by the app and the CLI |
| `crates/mmdcat` | Terminal renderer CLI and zsh widget |
| `specs/` | Feature specifications and plans |
| `scripts/` | Cleanup, tray icon generation and launch agent install |

## Getting Started

```bash
pnpm install
pnpm dev
```

For the native app:

```bash
pnpm tauri dev
```

## Checks

```bash
pnpm typecheck                                  # frontend
cd apps/desktop/src-tauri && cargo test         # native app
cd crates && cargo test --workspace             # mermaid-core and mmdcat
```

## Install the desktop app

Build the app, copy it to `/Applications`, and have it start at login:

```bash
pnpm tauri build --bundles app
cp -R apps/desktop/src-tauri/target/release/bundle/macos/*.app /Applications/
pnpm install:agent                              # remove: pnpm install:agent --uninstall
```

When replacing an installed build, quit the running app first and restart it through the launch agent rather than opening it directly, so only one instance holds the render socket:

```bash
launchctl kickstart -k gui/$(id -u)/dev.yoophi.mermaid-live
```

Right after the bundle is replaced, launchd may refuse one launch with a code signing error; the kickstart above starts it again.

## Terminal inline rendering

Select a Mermaid chart in a terminal and press **Ctrl-X v** to see it rendered in place, without switching windows. The chart is rasterised by the running desktop app, so the desktop app must be installed and running (see above).

```bash
cargo install --path crates/mmdcat
```

Then source the shell widget from `~/.zshrc`:

```sh
source /path/to/mermaid-live/crates/mmdcat/shell/mmdcat.zsh
```

To have charts appear as soon as they are copied, without selecting anything, run `mmdcat --watch` in a split.

Works in Ghostty, WezTerm and kitty, each of which needs one line of terminal configuration so that selecting text reaches the system clipboard. Run `mmdcat --probe` to check readiness - it reports terminal support, cell size, what the clipboard holds, and whether the render service is up. See [crates/mmdcat/README.md](crates/mmdcat/README.md) for usage and the per-terminal setup.

## Scripts

| Command | Description |
|---|---|
| `pnpm install:agent` | Install the launch agent that starts the app at login (`--uninstall` to remove) |
| `pnpm generate:tray-icon` | Regenerate the menu bar template icon at `apps/desktop/src-tauri/icons/tray.png` |
| `pnpm run clean` | Remove generated resources (see below) |

## Versioning

The project uses CalVer in `YYYY.M.#` form (for example `2026.8.1`). Development builds are identified as `YYYY.M.#-{short-commit-hash}[-dirty]`, injected at build time. See the CalVer section in [CLAUDE.md](CLAUDE.md) for the release rules.

## Cleanup

Remove development-generated resources: `node_modules`, `apps/desktop/dist`,
`apps/desktop/src-tauri/target`, `apps/desktop/src-tauri/gen`, and `crates/target`:

```bash
pnpm run clean              # add --dry-run to list what would be removed
```
