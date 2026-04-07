// Parse the AI brief markdown produced by the Rust ReportService into
// discrete sections that the PDF report can weave inline with its own
// data. The brief is always assembled with stable `## Diagnosis`,
// `## Priority Actions`, `## Pillar Health`, `## Next Steps` headings
// (see src-tauri/.../report_service.rs::assemble_brief and
// brief_builder::build_static_brief).

export interface BriefSections {
  diagnosis: string;
  /** Lowercased pattern name → AI paragraph for that pattern. */
  priorityByPattern: Map<string, string>;
  pillarHealth: string;
  nextSteps: string;
}

const EMPTY: BriefSections = {
  diagnosis: "",
  priorityByPattern: new Map(),
  pillarHealth: "",
  nextSteps: "",
};

export function parseBrief(md: string | undefined | null): BriefSections {
  if (!md || !md.trim()) return EMPTY;

  const sections: Record<string, string> = {};
  let current: string | null = null;
  let buf: string[] = [];

  const flush = () => {
    if (current !== null) sections[current] = buf.join("\n").trim();
    buf = [];
  };

  for (const line of md.split("\n")) {
    const m = line.match(/^##\s+(.+?)\s*$/);
    if (m) {
      flush();
      current = m[1].toLowerCase();
    } else if (current !== null) {
      buf.push(line);
    }
  }
  flush();

  // Priority Actions: split into per-pattern paragraphs.
  // Format from report_service.rs is:
  //   **Pattern Name** (42% of pages)\n<one or more sentences>\n\n
  // Static fallback uses:
  //   **Pattern Name** — affects 42% of pages. <recommendation>\n\n
  const priorityByPattern = new Map<string, string>();
  const priorityRaw = sections["priority actions"] ?? "";
  if (priorityRaw) {
    const blocks = priorityRaw.split(/\n\s*\n/);
    for (const block of blocks) {
      const m = block.match(/^\*\*(.+?)\*\*\s*(?:\((.*?)\)|—\s*[^.]*\.)?\s*([\s\S]*)$/);
      if (m) {
        const name = m[1].trim().toLowerCase();
        const body = (m[3] ?? "").trim();
        if (name && body) priorityByPattern.set(name, body);
      }
    }
  }

  return {
    diagnosis:    sections["diagnosis"] ?? "",
    priorityByPattern,
    pillarHealth: sections["pillar health"] ?? "",
    nextSteps:    sections["next steps"] ?? "",
  };
}

/** Pull the first sentence (or first ~140 chars) from a paragraph. */
export function firstSentence(s: string): string {
  if (!s) return "";
  const clean = s.replace(/\s+/g, " ").trim();
  const m = clean.match(/^(.+?[.!?])(\s|$)/);
  return (m ? m[1] : clean.slice(0, 140)).trim();
}

/**
 * Cap a paragraph at the first N sentences. Local models loop and we
 * shouldn't trust them to self-limit; this gives every brief section
 * a hard upper bound so cards never overflow into clipped territory.
 */
export function capSentences(s: string, n: number): string {
  if (!s) return "";
  const clean = s.replace(/\s+/g, " ").trim();
  // Match runs of "non-terminal chars + terminator + space".
  const re = /[^.!?]+[.!?]+(?:\s+|$)/g;
  const out: string[] = [];
  let m: RegExpExecArray | null;
  while ((m = re.exec(clean)) !== null && out.length < n) {
    out.push(m[0].trim());
  }
  if (out.length === 0) return clean;
  return out.join(" ");
}

/** True when a section is just a bullet list of pillar scores (no prose). */
export function isJustScoreList(s: string): boolean {
  if (!s) return true;
  const lines = s.split("\n").map((l) => l.trim()).filter(Boolean);
  if (lines.length === 0) return true;
  return lines.every((l) => /^[-*•]?\s*(Technical|Content|Performance|Accessibility)\s*[:\-—]\s*\d+/i.test(l));
}
