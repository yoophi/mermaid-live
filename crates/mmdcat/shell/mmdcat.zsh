# Renders the current clipboard contents as a Mermaid chart, inline.
#
# Terminals copy the selection to the clipboard as you select it, so the widget
# needs no access to the terminal's selection API and works the same in
# Ghostty, WezTerm and kitty.
#
# Usage: source this file from ~/.zshrc, then select a chart and press Ctrl-X v.
#
# Set MMDCAT_KEY before sourcing to bind a different key. The default avoids
# every stock zsh binding, including ^Xm, which is zsh's _most_recent_file.

: ${MMDCAT_KEY:='^Xv'}

# Prefer an installed binary; fall back to a build in the repository.
if [[ -z ${MMDCAT_BIN:-} ]]; then
  if (( $+commands[mmdcat] )); then
    MMDCAT_BIN=$commands[mmdcat]
  else
    _mmdcat_shell_dir="${${(%):-%x}:A:h}"
    for _mmdcat_candidate in \
      "$_mmdcat_shell_dir/../../target/release/mmdcat" \
      "$_mmdcat_shell_dir/../../target/debug/mmdcat"
    do
      if [[ -x $_mmdcat_candidate ]]; then
        MMDCAT_BIN=${_mmdcat_candidate:A}
        break
      fi
    done
    unset _mmdcat_shell_dir _mmdcat_candidate
  fi
fi

_mmdcat_render_selection() {
  emulate -L zsh

  if [[ -z ${MMDCAT_BIN:-} || ! -x $MMDCAT_BIN ]]; then
    zle -M "mmdcat: not installed - run 'cargo install --path crates/mmdcat'"
    return 1
  fi

  # Let the line editor release the display before anything is drawn over it.
  zle -I

  "$MMDCAT_BIN" </dev/null >/dev/tty 2>/dev/tty

  zle reset-prompt
}

zle -N mmdcat-render-selection _mmdcat_render_selection

# Bind in every editing keymap rather than just the current one.
#
# `bindkey -v` and `bindkey -e` re-point `main` at another keymap, so a binding
# made only in `main` disappears when something switches modes later - which is
# exactly what happens when this file is sourced before oh-my-zsh's vi-mode
# plugin. Binding each keymap directly also means the key works in vi command
# mode, not only while inserting.
for _mmdcat_keymap in emacs viins vicmd; do
  if [[ -n ${(M)$(bindkey -l):#$_mmdcat_keymap} ]]; then
    bindkey -M "$_mmdcat_keymap" "$MMDCAT_KEY" mmdcat-render-selection
  fi
done
unset _mmdcat_keymap
