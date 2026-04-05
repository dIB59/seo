"use client";

import { useState, useMemo } from "react";
import { ChevronDown, ChevronUp, Zap, AlertCircle } from "lucide-react";
import { Textarea } from "@/src/components/ui/textarea";
import { Badge } from "@/src/components/ui/badge";

// ---------------------------------------------------------------------------
// Default sample HTML — covers the most common extractor targets
// ---------------------------------------------------------------------------
const DEFAULT_HTML = `<!DOCTYPE html>
<html>
<head>
  <title>Sample Page Title</title>
  <meta name="description" content="A sample meta description." />
  <meta property="og:title" content="OG Title Here" />
  <meta property="og:image" content="https://example.com/image.jpg" />
  <link rel="canonical" href="https://example.com/page" />
  <link rel="alternate" hreflang="en-US" href="https://example.com/en/" />
  <link rel="alternate" hreflang="fr" href="https://example.com/fr/" />
  <script type="application/ld+json">{"@type":"Article","name":"Sample"}</script>
</head>
<body>
  <h1>Main Page Heading</h1>
  <h2>Section One</h2>
  <p>First paragraph with some content.</p>
  <h2>Section Two</h2>
  <p>Second paragraph here.</p>
  <a href="/link-one">Internal Link</a>
  <a href="https://external.com">External Link</a>
  <img src="hero.jpg" alt="Hero image description" />
</body>
</html>`;

// ---------------------------------------------------------------------------
// Core: parse HTML, run selector, produce highlighted source + extracted values
// ---------------------------------------------------------------------------

// Sentinel chars that survive HTML escaping (&, <, >, ")
const OPEN = "\x01";
const CLOSE = "\x02";

interface MatchResult {
  /** HTML-escaped source with <mark> tags injected around matches */
  highlighted: string;
  matchCount: number;
  values: string[];
  selectorError: boolean;
}

function buildResult(
  html: string,
  selector: string,
  attribute: string | null,
): MatchResult {
  const empty: MatchResult = {
    highlighted: "",
    matchCount: 0,
    values: [],
    selectorError: false,
  };

  if (!selector.trim()) return empty;

  let doc: Document;
  try {
    doc = new DOMParser().parseFromString(html, "text/html");
  } catch {
    return empty;
  }

  let matched: Element[];
  try {
    matched = Array.from(doc.querySelectorAll(selector));
  } catch {
    return { ...empty, selectorError: true };
  }

  // Extract values
  const values = matched
    .map((el) =>
      attribute
        ? (el.getAttribute(attribute) ?? null)
        : (el.textContent?.trim() ?? null),
    )
    .filter((v): v is string => v !== null && v.length > 0);

  // Build highlighted source string
  // Step 1: start from the browser-normalised DOM serialisation
  let source = doc.documentElement.outerHTML;

  // Step 2: wrap each matched element's outerHTML with sentinels.
  //         Process in document order — after wrapping element N the sentinel
  //         chars make the already-wrapped region unsearchable, so indexOf
  //         naturally skips it and lands on the next raw occurrence.
  for (const el of matched) {
    const outer = el.outerHTML;
    if (!outer) continue;
    const idx = source.indexOf(outer);
    if (idx !== -1) {
      source =
        source.slice(0, idx) + OPEN + outer + CLOSE + source.slice(idx + outer.length);
    }
  }

  // Step 3: HTML-escape (sentinels are not &/<>/", so they survive)
  const escaped = source
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");

  // Step 4: convert sentinels to actual <mark> tags (safe to inject)
  const highlighted = escaped
    .replace(/\x01/g, '<mark class="seo-hl">')
    .replace(/\x02/g, "</mark>");

  return { highlighted, matchCount: matched.length, values, selectorError: false };
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

interface Props {
  selector: string;
  attribute: string | null;
}

export function SelectorLivePreview({ selector, attribute }: Props) {
  const [open, setOpen] = useState(true);
  const [sampleHtml, setSampleHtml] = useState(DEFAULT_HTML);

  const result = useMemo(
    () => buildResult(sampleHtml, selector, attribute),
    [sampleHtml, selector, attribute],
  );

  const matchBadge = selector && !result.selectorError && (
    <Badge
      variant={result.matchCount > 0 ? "default" : "outline"}
      className="text-xs h-5 ml-1"
    >
      {result.matchCount === 0
        ? "no match"
        : `${result.matchCount} match${result.matchCount !== 1 ? "es" : ""}`}
    </Badge>
  );

  return (
    <div className="rounded-lg border border-border/60 overflow-hidden">
      {/* ------------------------------------------------------------------ */}
      {/* Collapse header                                                     */}
      {/* ------------------------------------------------------------------ */}
      <button
        type="button"
        className="w-full flex items-center justify-between px-4 py-2.5 text-sm font-medium bg-muted/20 hover:bg-muted/40 transition-colors"
        onClick={() => setOpen((o) => !o)}
      >
        <span className="flex items-center gap-1.5">
          <Zap className="h-3.5 w-3.5 text-primary" />
          Live preview
          {matchBadge}
          {result.selectorError && (
            <Badge variant="destructive" className="text-xs h-5 ml-1">
              invalid selector
            </Badge>
          )}
        </span>
        {open ? (
          <ChevronUp className="h-3.5 w-3.5 text-muted-foreground" />
        ) : (
          <ChevronDown className="h-3.5 w-3.5 text-muted-foreground" />
        )}
      </button>

      {/* ------------------------------------------------------------------ */}
      {/* Body                                                                */}
      {/* ------------------------------------------------------------------ */}
      {open && (
        <div className="border-t border-border/60">
          <div className="grid grid-cols-2 divide-x divide-border/60">
            {/* Left: editable HTML */}
            <div className="p-3 space-y-1.5">
              <p className="text-xs font-medium text-muted-foreground">
                Sample HTML — edit to test your selector
              </p>
              <Textarea
                className="font-mono text-xs h-64 resize-none bg-muted/20 leading-relaxed"
                value={sampleHtml}
                onChange={(e) => setSampleHtml(e.target.value)}
                spellCheck={false}
              />
            </div>

            {/* Right: highlighted source + extracted values */}
            <div className="p-3 space-y-2 flex flex-col">
              <p className="text-xs font-medium text-muted-foreground">
                Matching elements are{" "}
                <mark className="bg-primary/25 text-foreground rounded px-0.5 not-italic">
                  highlighted
                </mark>
              </p>

              {/* Source code pane */}
              <div className="flex-1 overflow-auto rounded-md bg-muted/20 border border-border/40 h-44">
                {!selector || result.selectorError ? (
                  <p className="text-xs text-muted-foreground p-3">
                    {result.selectorError
                      ? "Invalid CSS selector."
                      : "Type a selector above to see matches here."}
                  </p>
                ) : (
                  /* dangerouslySetInnerHTML is safe here:
                     - This is a Tauri desktop app (no external HTML served)
                     - The user supplies their own sample HTML
                     - We HTML-escape the entire source before injecting;
                       the only raw HTML we inject are the <mark> wrappers we
                       control ourselves */
                  <pre
                    className="text-xs p-3 whitespace-pre-wrap break-all leading-relaxed
                               [&_mark.seo-hl]:bg-primary/25
                               [&_mark.seo-hl]:text-foreground
                               [&_mark.seo-hl]:rounded
                               [&_mark.seo-hl]:outline
                               [&_mark.seo-hl]:outline-1
                               [&_mark.seo-hl]:outline-primary/60"
                    dangerouslySetInnerHTML={{ __html: result.highlighted }}
                  />
                )}
              </div>

              {/* Extracted values */}
              {result.values.length > 0 && (
                <div className="space-y-1">
                  <p className="text-xs font-medium text-muted-foreground">
                    {attribute
                      ? `Extracted "${attribute}" value${result.values.length > 1 ? "s" : ""}:`
                      : `Extracted text${result.values.length > 1 ? " (all matches)" : ""}:`}
                  </p>
                  <div className="flex flex-wrap gap-1.5">
                    {result.values.map((v, i) => (
                      <code
                        key={i}
                        className="text-xs bg-primary/10 text-foreground border border-primary/20 rounded px-1.5 py-0.5 max-w-full truncate"
                        title={v}
                      >
                        {v}
                      </code>
                    ))}
                  </div>
                </div>
              )}

              {/* No match hint */}
              {selector && !result.selectorError && result.matchCount === 0 && (
                <p className="text-xs text-muted-foreground flex items-center gap-1">
                  <AlertCircle className="h-3 w-3 shrink-0" />
                  No match in sample HTML. Edit the HTML on the left or adjust your selector.
                </p>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
