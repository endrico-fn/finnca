const REF_RE = /\[Ref:\s*([^\]]+)\]/;
const REF_STRIP_RE = /\[Ref:\s*[^\]]+\]\s*/;
const DUE_RE = /\[Due:\s*([^\]]+)\]/;
const DUE_STRIP_RE = /\[Due:\s*[^\]]+\]\s*/;
const SETTLED_STRIP_RE = /\[Settled\]\s*/;
const SETTLED_TRIM_RE = /\s*\[Settled\]/i;
const SETTLED_GLOBAL_RE = /\[Settled\]/g;

export interface ParsedNoteTags {
  ref: string;
  dueDate: string;
  settled: boolean;
  cleanNotes: string;
}

export function parseNoteTags(input: {
  notes?: string | null;
  reference_no?: string | null;
  due_date?: string | null;
}): ParsedNoteTags {
  const notes = input.notes ?? '';
  const ref = input.reference_no ?? notes.match(REF_RE)?.[1] ?? '';
  const afterRef = notes.replace(REF_STRIP_RE, '');
  const rawDue = input.due_date ?? afterRef.match(DUE_RE)?.[1] ?? '';
  const settled = (input.due_date?.includes('[Settled]') || notes.includes('[Settled]')) ?? false;
  return {
    ref,
    dueDate: rawDue.replace(SETTLED_TRIM_RE, '').trim(),
    settled,
    cleanNotes: afterRef.replace(DUE_STRIP_RE, '').replace(SETTLED_STRIP_RE, '').trim(),
  };
}

export function stripSettledMarker(value: string): string {
  return value.replace(SETTLED_TRIM_RE, '').trim();
}

export function serializeNoteTags(cleanNotes: string, settled: boolean): string | null {
  const base = cleanNotes.replace(SETTLED_GLOBAL_RE, '').trim() || '';
  if (!settled) return base || null;
  return base ? `${base} [Settled]` : '[Settled]';
}
