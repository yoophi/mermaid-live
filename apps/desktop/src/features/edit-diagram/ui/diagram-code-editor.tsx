import { useEffect, useRef } from "react";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { markdown } from "@codemirror/lang-markdown";
import { EditorState } from "@codemirror/state";
import {
  drawSelection,
  EditorView,
  highlightActiveLine,
  keymap,
  lineNumbers,
} from "@codemirror/view";

interface DiagramCodeEditorProps {
  value: string;
  onChange: (value: string) => void;
}

export function DiagramCodeEditor({ value, onChange }: DiagramCodeEditorProps) {
  const hostRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const onChangeRef = useRef(onChange);

  useEffect(() => {
    onChangeRef.current = onChange;
  }, [onChange]);

  useEffect(() => {
    if (!hostRef.current) {
      return;
    }

    const view = new EditorView({
      parent: hostRef.current,
      state: EditorState.create({
        doc: value,
        extensions: [
          lineNumbers(),
          history(),
          drawSelection(),
          highlightActiveLine(),
          markdown(),
          keymap.of([...defaultKeymap, ...historyKeymap]),
          EditorView.lineWrapping,
          EditorView.updateListener.of((update) => {
            if (update.docChanged) {
              onChangeRef.current(update.state.doc.toString());
            }
          }),
          EditorView.theme({
            "&": {
              height: "100%",
              background: "transparent",
              color: "var(--foreground)",
              fontSize: "14px",
            },
            ".cm-scroller": {
              fontFamily:
                "'SFMono-Regular', 'Cascadia Code', 'Liberation Mono', monospace",
              lineHeight: "1.62",
              overflow: "auto",
            },
            ".cm-gutters": {
              background: "color-mix(in oklab, var(--muted), transparent 36%)",
              color: "var(--muted-foreground)",
              borderRight: "1px solid var(--border)",
            },
            ".cm-activeLine": {
              background: "color-mix(in oklab, var(--accent), transparent 82%)",
            },
            ".cm-activeLineGutter": {
              background: "color-mix(in oklab, var(--accent), transparent 78%)",
              color: "var(--foreground)",
            },
            ".cm-content": {
              padding: "18px 0",
            },
            ".cm-line": {
              padding: "0 18px",
            },
            ".cm-focused": {
              outline: "none",
            },
          }),
        ],
      }),
    });

    viewRef.current = view;

    return () => {
      view.destroy();
      viewRef.current = null;
    };
  }, []);

  useEffect(() => {
    const view = viewRef.current;

    if (!view || view.state.doc.toString() === value) {
      return;
    }

    view.dispatch({
      changes: {
        from: 0,
        to: view.state.doc.length,
        insert: value,
      },
    });
  }, [value]);

  return <div ref={hostRef} className="h-full min-h-0" />;
}
