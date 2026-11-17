import { useLayoutEffect, useRef } from "react";

interface Props {
  value: string;
  onChange: (value: string) => void;
}

export function YamlEditor({ value, onChange }: Props) {
  const editor = useRef<HTMLTextAreaElement>(null);
  const selection = useRef<{ start: number; end: number; direction: "forward" | "backward" | "none"; scrollTop: number; scrollLeft: number } | null>(null);

  useLayoutEffect(() => {
    if (!selection.current || !editor.current) return;
    const { start, end, direction, scrollTop, scrollLeft } = selection.current;
    editor.current.setSelectionRange(start, end, direction);
    editor.current.scrollTop = scrollTop;
    editor.current.scrollLeft = scrollLeft;
    selection.current = null;
  }, [value]);

  return (
    <textarea
      ref={editor}
      className="node-properties"
      rows={8}
      spellCheck={false}
      value={value}
      onChange={event => onChange(event.target.value)}
      onKeyDown={event => {
        if (event.key !== "Tab" || event.ctrlKey || event.altKey || event.metaKey || event.nativeEvent.isComposing) return;
        event.preventDefault();
        const textarea = event.currentTarget;
        const { selectionStart: start, selectionEnd: end, selectionDirection: direction, scrollTop, scrollLeft } = textarea;
        const edits: { start: number; remove: number; insert: string }[] = [];
        if (start === end && !event.shiftKey) {
          edits.push({ start, remove: 0, insert: "  " });
        } else {
          const blockStart = start === 0 ? 0 : value.lastIndexOf("\n", start - 1) + 1;
          const lastPosition = end > start && value[end - 1] === "\n" ? end - 1 : end;
          const nextNewline = value.indexOf("\n", lastPosition);
          const blockEnd = nextNewline === -1 ? value.length : nextNewline;
          let offset = blockStart;
          for (const line of value.slice(blockStart, blockEnd).split("\n")) {
            const remove = event.shiftKey ? (line.startsWith("\t") ? 1 : line.match(/^ {0,2}/)![0].length) : 0;
            edits.push({ start: offset, remove, insert: event.shiftKey ? "" : "  " });
            offset += line.length + 1;
          }
        }
        let next = value;
        for (const edit of [...edits].reverse()) {
          next = next.slice(0, edit.start) + edit.insert + next.slice(edit.start + edit.remove);
        }
        if (next === value) return;
        const move = (position: number) => position + edits.reduce((delta, edit) =>
          delta + (position < edit.start ? 0 : edit.insert.length - Math.min(edit.remove, position - edit.start)), 0);
        selection.current = { start: move(start), end: move(end), direction, scrollTop, scrollLeft };
        onChange(next);
      }}
    />
  );
}
