# Mermaid Live

A Tauri desktop app for editing Mermaid diagrams with a live preview.

## Stack

- pnpm monorepo
- Tauri
- React + Vite + TypeScript
- shadcn/ui-style shared components
- Feature-Sliced Design frontend
- Hexagonal architecture native layer

## Getting Started

```bash
pnpm install
pnpm dev
```

For the native app:

```bash
pnpm tauri dev
```

## Terminal inline rendering

Select a Mermaid chart in a terminal and press **Ctrl-X v** to see it rendered in place, without switching windows. The chart is rasterised by the running desktop app, so the terminal image uses the same Mermaid version and theme as the preview.

```bash
cargo install --path crates/mmdcat
```

Then source the shell widget from `~/.zshrc`:

```sh
source /path/to/mermaid-live/crates/mmdcat/shell/mmdcat.zsh
```

To have charts appear as soon as they are copied, without selecting anything, run `mmdcat --watch` in a split.

The renderer is the desktop app itself, which lives in the menu bar with no window. Install it and have it start at login:

```bash
pnpm tauri build --bundles app
cp -R apps/desktop/src-tauri/target/release/bundle/macos/*.app /Applications/
pnpm install:agent
```

Works in Ghostty, WezTerm and kitty, each of which needs one line of terminal configuration so that selecting text reaches the system clipboard. Run `mmdcat --probe` to check readiness - it reports terminal support, cell size, what the clipboard holds, and whether the render service is up. See [crates/mmdcat/README.md](crates/mmdcat/README.md) for the per-terminal setup.

## Cleanup

Remove development-generated resources such as `node_modules`, `apps/desktop/dist`,
`apps/desktop/src-tauri/target`, and `crates/target`:

```bash
pnpm run clean
```
