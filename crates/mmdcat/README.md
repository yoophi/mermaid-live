# mmdcat

Renders a Mermaid chart inline in the terminal using the kitty graphics protocol.

The chart is rasterised by the **Mermaid Live desktop app**, which must be running. It is configured to live in the menu bar with no window and no Dock icon, so "running" is unobtrusive; `pnpm install:agent` starts it at login. That keeps one Mermaid version and one theme definition for both the desktop preview and the terminal. Chart detection, the raster size contract and the socket protocol are shared with the app through the `mermaid-core` crate, so the two ends cannot drift.

## Requirements

The desktop app provides the renderer, so it has to be running. Install it once and have it start at login:

```bash
pnpm tauri build --bundles app
cp -R apps/desktop/src-tauri/target/release/bundle/macos/*.app /Applications/
pnpm install:agent                                    # remove: --uninstall
```

It runs as a menu bar accessory: a tray icon, no window, no Dock icon. Open a window from the tray menu when you want one.

If a stale bundle is left in `/Applications`, `mmdcat` will launch that one, it will never open a socket, and the CLI will wait and then give up. Keep the bundle current or delete it and use `pnpm tauri dev`.

Rendering needs a GUI session, so this does not work over plain SSH.

## Install

```bash
cargo install --path crates/mmdcat
```

That puts `mmdcat` on `PATH` via `~/.cargo/bin`. To work from a build instead:

```bash
cd crates && cargo build --release   # target/release/mmdcat
```

The shell widget finds an installed `mmdcat` first and falls back to the build in this repository, so `PATH` is not required for the keybinding.

## Test

```bash
cd crates && cargo test
```

## Usage

```bash
mmdcat                # render the clipboard contents
mmdcat -              # render from stdin
mmdcat chart.mmd      # render a file
mmdcat --watch        # stay open and show every chart copied to the clipboard
mmdcat --watch --clear  # …replacing the screen instead of appending
mmdcat --probe        # report terminal and render service readiness
```

Text that is not a Mermaid chart produces no output at all.

## Watch mode

`--watch` subscribes to the desktop app's clipboard watcher and shows each chart as it is copied, so nothing has to be selected or typed. It runs until interrupted, which makes it a good fit for a dedicated split.

Two things follow from subscribing to the app rather than watching the clipboard here:

- **The app's tray toggle also controls this.** If clipboard watching is turned off there, `--watch` stays quiet and says so at startup.
- **A chart shown this way is not staged in the tray.** You have already seen it, so no badge or notification is raised — and it cannot be reopened in a large window later.

A chart already shown is not shown again, including one you rendered earlier with a one-shot command. Resizing the terminal resubscribes at the new size, and a restarting app is reconnected to automatically.

## Shell integration

Add to `~/.zshrc`:

```sh
source /path/to/mermaid-live/crates/mmdcat/shell/mmdcat.zsh
```

Select a chart in the terminal and press **Ctrl-X v**.

## Terminal setup

Selection has to reach the clipboard, and the terminal has to support inline images.

### Ghostty

This line is **required** in `~/.config/ghostty/config`:

```
copy-on-select = clipboard
```

The default is `true`, which per Ghostty's own docs "will prefer to copy to the selection clipboard". Ghostty keeps a selection clipboard of its own on macOS for middle-click paste, and `pbpaste` cannot see it — so selecting text appears to do nothing. Only `clipboard` copies "to the selection clipboard as well as the system clipboard". Valid values are `false`, `true`, `clipboard`.

To keep the clipboard untouched instead, skip the setting and use Ghostty's own selection export:

```
keybind = super+shift+m=write_selection_file:paste,plain
```

That pastes a temp file path at the prompt; type `mmdcat ` first, then press the key, then Enter.

### kitty

`copy_on_select` defaults to `no`, so this line is **required** in `~/.config/kitty/kitty.conf`:

```
copy_on_select clipboard
```

kitty can also hand the selection over directly, without the clipboard:

```
map cmd+shift+m launch --type=overlay --stdin-source=@selection mmdcat -
```

### WezTerm

Kitty graphics support has varied across releases, so state it explicitly in `~/.wezterm.lua`:

```lua
config.enable_kitty_graphics = true
```

Selection already reaches the system clipboard: as of 20240203 every unmodified selection completes into `ClipboardAndPrimarySelection`. Verified with `wezterm show-keys`, which prints the effective mouse bindings:

```
Up { streak: 1, button: Left } -> CompleteSelectionOrOpenLinkAtMouseCursor(ClipboardAndPrimarySelection)
Up { streak: 2, button: Left } -> CompleteSelection(ClipboardAndPrimarySelection)
Up { streak: 3, button: Left } -> CompleteSelection(ClipboardAndPrimarySelection)
```

So no clipboard setting is required. To pin that against a future change to the default, restate the same bindings; keep `CompleteSelectionOrOpenLinkAtMouseCursor` on the single click so that clicking a hyperlink still works:

```lua
config.mouse_bindings = {
  {
    event = { Up = { streak = 1, button = 'Left' } },
    mods = 'NONE',
    action = wezterm.action.CompleteSelectionOrOpenLinkAtMouseCursor 'ClipboardAndPrimarySelection',
  },
  {
    event = { Up = { streak = 2, button = 'Left' } },
    mods = 'NONE',
    action = wezterm.action.CompleteSelection 'ClipboardAndPrimarySelection',
  },
  {
    event = { Up = { streak = 3, button = 'Left' } },
    mods = 'NONE',
    action = wezterm.action.CompleteSelection 'ClipboardAndPrimarySelection',
  },
}
```

Note that WezTerm reads `~/.wezterm.lua` before `~/.config/wezterm/wezterm.lua`; if both exist only the first is used.

## Environment variables

| Variable | Purpose |
|---|---|
| `MERMAID_LIVE_SOCKET` | Override the render socket path |
| `MERMAID_LIVE_APP` | Override the app name used to start the renderer |
| `MMDCAT_BIN` | Override the CLI path used by the zsh widget |
| `MMDCAT_KEY` | Bind a different key; set it before sourcing the widget |

## Notes

- **`mmdcat --probe` reports what the clipboard holds.** Selecting text does not reach the system clipboard on every terminal, and a stale clipboard produces no output at all - the same symptom as a key that never fired.
- **Very tall charts get small.** The image is fitted to the terminal so it never pushes the prompt off screen, so a long vertical `flowchart TD` ends up narrow. Widen the terminal, or open the chart in the desktop window instead.
- Images are transmitted with `t=d` (direct) because it is the only medium implemented by all three target terminals.
- Under `tmux` or `screen`, inline images need passthrough support in the multiplexer. `mmdcat --probe` reports when one is detected.
