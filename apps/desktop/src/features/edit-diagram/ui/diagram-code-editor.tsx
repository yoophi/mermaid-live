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
              background: "#c0c0c0",
              color: "#000",
              fontSize: "12px",
            },
            ".cm-scroller, .cm-scroller *": {
              fontFamily: "'Courier New', Courier, monospace",
            },
            ".cm-scroller": {
              lineHeight: "1.5",
              overflow: "auto",
            },
            ".cm-gutters": {
              background: "#c0c0c0",
              color: "#000",
              borderRight: "1px solid #808080",
              boxShadow: "inset -1px 0 #fff",
            },
            ".cm-activeLine": {
              background: "#d4d0c8",
            },
            ".cm-activeLineGutter": {
              background: "#d4d0c8",
              color: "#000",
            },
            ".cm-content": {
              padding: "12px 0",
            },
            ".cm-line": {
              padding: "0 12px",
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
