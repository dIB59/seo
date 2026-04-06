/**
 * SEO Insikt — PDF Report
 *
 * Design principles:
 *  • Every page shares the same dark top bar — consistent frame, no visual jump
 *  • Muted palette: colour is reserved for score numbers and severity dots only
 *  • No filled coloured boxes — severity expressed through a 2px left rule and coloured text
 *  • Generous but deliberate whitespace; pages are dense enough to not look empty
 *  • Score ring mirrors the app's ScoreRing component (SVG strokeDasharray)
 */
import React from "react";
import {
  Document,
  Page,
  View,
  Text,
  Svg,
  Circle,
  StyleSheet,
} from "@react-pdf/renderer";
import type { ReportData, DetectedPattern } from "@/src/bindings";
import { C, F, SP, scoreColor, scoreGrade, severityColor, pillarColor, stripMd } from "./theme";

// ── Base stylesheet ───────────────────────────────────────────────────────────

const s = StyleSheet.create({
  page: {
    backgroundColor: C.pageBg,
    fontFamily: F.regular,
    color: C.text,
    fontSize: 9,
  },

  // Page-level dark top bar
  bar: {
    backgroundColor: C.ink,
    paddingHorizontal: SP.page,
    paddingVertical: 10,
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
  },
  barLeft:  { fontSize: 7, color: C.inkMuted, fontFamily: F.regular, letterSpacing: 0.6, textTransform: "uppercase" },
  barRight: { fontSize: 7, color: C.inkMuted, fontFamily: F.regular, letterSpacing: 0.4 },

  // Body area beneath the bar
  body: {
    flex: 1,
    paddingHorizontal: SP.page,
    paddingTop: SP.gap.xl,
    paddingBottom: 44,   // leave room for footer
  },

  // Section label (micro uppercase above hairline)
  sectionLabel: {
    fontSize: 7,
    fontFamily: F.regular,
    color: C.faint,
    textTransform: "uppercase",
    letterSpacing: 1,
    marginBottom: 5,
  },

  // Hairline divider
  hr: { borderBottomWidth: 0.5, borderBottomColor: C.border },

  // Text scale — consistent 1.4× ratio steps
  h1: { fontSize: 20, fontFamily: F.bold, color: C.text, lineHeight: 1.2 },
  h2: { fontSize: 13, fontFamily: F.bold, color: C.text, lineHeight: 1.25 },
  h3: { fontSize: 10, fontFamily: F.bold, color: C.secondary, lineHeight: 1.3 },
  body1: { fontSize: 9, lineHeight: 1.6, color: C.text },
  body2: { fontSize: 8.5, lineHeight: 1.55, color: C.secondary },
  caption: { fontSize: 7.5, lineHeight: 1.4, color: C.muted },
  mono:    { fontFamily: F.mono, fontSize: 7.5, color: C.muted },

  // Layout helpers
  row:     { flexDirection: "row", alignItems: "center" },
  center:  { alignItems: "center" },
  flex1:   { flex: 1 },
});

// ── Shared primitives ─────────────────────────────────────────────────────────

/** Tiny colour dot for severity / status indicators */
function Dot({ color, size = 5 }: { color: string; size?: number }) {
  return (
    <View style={{ width: size, height: size, borderRadius: size / 2, backgroundColor: color, flexShrink: 0 }} />
  );
}

/** Score ring — direct port of the app's ScoreRing SVG logic */
function ScoreRing({ score, size = 64 }: { score: number; size?: number }) {
  const sw   = size * 0.1;
  const r    = (size - sw) / 2;
  const cx   = size / 2;
  const circ = 2 * Math.PI * r;
  const pct  = Math.max(0, Math.min(100, score));
  const dash = circ - (circ * pct) / 100;
  const col  = scoreColor(score);

  return (
    <View style={{ width: size, height: size, position: "relative", alignItems: "center", justifyContent: "center" }}>
      <Svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} style={{ position: "absolute" }}>
        <Circle cx={cx} cy={cx} r={r} strokeWidth={sw} stroke={C.border} fill="none" />
        <Circle
          cx={cx} cy={cx} r={r}
          strokeWidth={sw} stroke={col} fill="none"
          strokeDasharray={`${circ} ${circ}`}
          strokeDashoffset={dash}
          strokeLinecap="round"
          transform={`rotate(-90 ${cx} ${cx})`}
        />
      </Svg>
      <View style={{ alignItems: "center" }}>
        <Text style={{ fontFamily: F.bold, fontSize: size * 0.28, color: col }}>{score}</Text>
        <Text style={{ fontSize: size * 0.13, color: C.faint }}>/100</Text>
      </View>
    </View>
  );
}

/** Page top bar — appears on every page */
function PageBar({ left, right }: { left: string; right?: string }) {
  return (
    <View style={s.bar} fixed>
      <Text style={s.barLeft}>{left}</Text>
      {right && <Text style={s.barRight}>{right}</Text>}
    </View>
  );
}

/** Sticky footer with domain + page number */
function PageFooter({ domain }: { domain: string }) {
  return (
    <View
      fixed
      style={{
        position: "absolute",
        bottom: 18,
        left: SP.page,
        right: SP.page,
        flexDirection: "row",
        justifyContent: "space-between",
        borderTopWidth: 0.5,
        borderTopColor: C.border,
        paddingTop: 5,
      }}
    >
      <Text style={{ fontSize: 7, color: C.faint }}>{domain}</Text>
      <Text
        style={{ fontSize: 7, color: C.faint }}
        render={({ pageNumber, totalPages }) => `${pageNumber} of ${totalPages}`}
      />
    </View>
  );
}

/** Section heading: small uppercase label + hairline */
function Section({ label, mt = SP.gap.xl }: { label: string; mt?: number }) {
  return (
    <View style={{ marginTop: mt }}>
      <Text style={s.sectionLabel}>{label}</Text>
      <View style={s.hr} />
      <View style={{ marginTop: SP.gap.md }} />
    </View>
  );
}

// ── Cover Page ────────────────────────────────────────────────────────────────

function CoverPage({ data, date }: { data: ReportData; date: string }) {
  const domain = data.url.replace(/^https?:\/\//, "").replace(/\/$/, "");
  const col    = scoreColor(data.seoScore);

  return (
    <Page size="A4" style={s.page}>
      {/* Top bar */}
      <PageBar left="SEO Insikt · Analysis Report" right={date} />

      {/* Hero */}
      <View style={{ paddingHorizontal: SP.page, paddingTop: SP.gap.xxl }}>
        {/* Domain */}
        <Text style={{ fontSize: 7, color: C.faint, marginBottom: 6, textTransform: "uppercase", letterSpacing: 1 }}>
          Website
        </Text>
        <Text style={{ fontFamily: F.bold, fontSize: 20, color: C.text, marginBottom: SP.gap.xxl }}>
          {domain}
        </Text>

        {/* Score + grade row */}
        <View style={{ ...s.row, gap: SP.gap.xl, marginBottom: SP.gap.xxl, alignItems: "flex-start" }}>
          <ScoreRing score={data.seoScore} size={88} />
          <View style={{ flex: 1 }}>
            <Text style={{ fontFamily: F.bold, fontSize: 18, color: col, marginBottom: 4 }}>
              {scoreGrade(data.seoScore)}
            </Text>
            <Text style={{ ...s.body2, marginBottom: SP.gap.sm }}>
              {data.totalPages} page{data.totalPages !== 1 ? "s" : ""} analysed
              {" · "}
              {data.totalIssues} issue{data.totalIssues !== 1 ? "s" : ""} detected
            </Text>
            {/* Health */}
            <View style={{ ...s.row, gap: SP.gap.xl, marginTop: 4 }}>
              {[
                { label: "Sitemap",    ok: data.sitemapFound },
                { label: "Robots.txt", ok: data.robotsTxtFound },
              ].map((h) => (
                <View key={h.label} style={{ ...s.row, gap: SP.gap.xs }}>
                  <Dot color={h.ok ? C.success : C.critical} size={6} />
                  <Text style={s.caption}>{h.label} {h.ok ? "found" : "missing"}</Text>
                </View>
              ))}
            </View>
          </View>
        </View>

        {/* Hairline */}
        <View style={s.hr} />

        {/* Key numbers row */}
        <View style={{ flexDirection: "row", marginTop: SP.gap.xl }}>
          {[
            { n: data.totalPages,       label: "Pages Analysed", color: C.text },
            { n: data.totalIssues,      label: "Total Issues",   color: C.text },
            { n: data.criticalIssues,   label: "Critical",       color: C.critical },
            { n: data.warningIssues,    label: "Warnings",       color: C.warning },
          ].map((m, i) => (
            <View
              key={m.label}
              style={{
                flex: 1,
                paddingRight: SP.gap.xl,
                borderRightWidth: i < 3 ? 0.5 : 0,
                borderRightColor: C.border,
                marginRight: i < 3 ? SP.gap.xl : 0,
              }}
            >
              <Text style={{ fontFamily: F.mono, fontSize: 24, color: m.color }}>
                {m.n}
              </Text>
              <Text style={{ ...s.caption, marginTop: 2 }}>{m.label}</Text>
            </View>
          ))}
        </View>

        {/* Pillar summary */}
        <View style={{ ...s.hr, marginTop: SP.gap.xl }} />
        <View style={{ flexDirection: "row", marginTop: SP.gap.xl, gap: SP.gap.xxl }}>
          {[
            ["Technical",     data.pillarScores.technical],
            ["Content",       data.pillarScores.content],
            ["Performance",   data.pillarScores.performance],
            ["Accessibility", data.pillarScores.accessibility],
          ].map(([label, score]) => {
            const n = score as number;
            return (
              <View key={label as string} style={{ flex: 1, alignItems: "center" }}>
                <ScoreRing score={Math.round(n)} size={44} />
                <View style={{ ...s.row, gap: SP.gap.xs, marginTop: 5 }}>
                  <Dot color={pillarColor(label as string)} size={4} />
                  <Text style={{ fontSize: 7, color: C.muted }}>{label as string}</Text>
                </View>
              </View>
            );
          })}
        </View>
      </View>

      <PageFooter domain={domain} />
    </Page>
  );
}

// ── Findings Page ─────────────────────────────────────────────────────────────

function PatternCard({ dp }: { dp: DetectedPattern }) {
  const col = severityColor(dp.pattern.severity);
  const pct = Math.round(dp.prevalence * 100);

  return (
    <View
      wrap={false}
      style={{
        marginBottom: SP.gap.xl,
        paddingLeft: SP.gap.md,
        borderLeftWidth: 2,
        borderLeftColor: col,
      }}
    >
      {/* Title + percentage */}
      <View style={{ ...s.row, justifyContent: "space-between", marginBottom: 3 }}>
        <Text style={{ fontFamily: F.bold, fontSize: 10, color: C.text, flex: 1 }}>
          {dp.pattern.name}
        </Text>
        <View style={{ ...s.row, gap: 4 }}>
          <Text style={{ fontFamily: F.bold, fontSize: 9, color: col }}>{pct}%</Text>
          <Text style={s.caption}>of pages</Text>
        </View>
      </View>

      {/* Description */}
      <Text style={{ ...s.body2, marginBottom: 5 }}>
        {dp.pattern.description}
      </Text>

      {/* Metadata row */}
      <View style={{ ...s.row, gap: SP.gap.xl, marginBottom: 5 }}>
        <Text style={s.mono}>{dp.affectedPages}/{dp.totalPages} pages</Text>
        <Text style={s.mono}>Impact: {dp.pattern.businessImpact}</Text>
        <Text style={s.mono}>Effort: {dp.pattern.fixEffort}</Text>
      </View>

      {/* Recommendation */}
      <View style={{
        backgroundColor: C.surface,
        borderRadius: 3,
        padding: SP.gap.sm,
        borderWidth: 0.5,
        borderColor: C.border,
        marginBottom: dp.sampleUrls.length > 0 ? 5 : 0,
      }}>
        <Text style={{ fontSize: 7, color: C.faint, marginBottom: 2, textTransform: "uppercase", letterSpacing: 0.5 }}>
          Recommendation
        </Text>
        <Text style={{ fontSize: 8.5, color: C.text, lineHeight: 1.45 }}>
          {dp.pattern.recommendation}
        </Text>
      </View>

      {/* Sample URLs */}
      {dp.sampleUrls.length > 0 && (
        <View>
          {dp.sampleUrls.slice(0, 3).map((u) => (
            <Text key={u} style={{ ...s.mono, marginBottom: 1 }}>
              {u.length > 74 ? u.slice(0, 71) + "…" : u}
            </Text>
          ))}
        </View>
      )}
    </View>
  );
}

function FindingsPage({ data }: { data: ReportData }) {
  const domain = data.url.replace(/^https?:\/\//, "").replace(/\/$/, "");
  const grouped: Partial<Record<string, DetectedPattern[]>> = {};
  for (const dp of data.detectedPatterns) {
    (grouped[dp.pattern.severity] ??= []).push(dp);
  }

  return (
    <Page size="A4" style={s.page}>
      <PageBar left={`${domain} · Findings`} right="SEO Insikt" />
      <View style={s.body}>
        <Text style={s.h2}>Detected Patterns</Text>
        <Text style={{ ...s.caption, marginTop: 4, marginBottom: SP.gap.xl }}>
          Sorted by priority score — highest impact and lowest effort first.
        </Text>

        {["critical", "warning", "suggestion"].map((sev) => {
          const items = grouped[sev] ?? [];
          if (items.length === 0) return null;
          return (
            <View key={sev}>
              {/* Group label */}
              <View style={{ ...s.row, gap: SP.gap.sm, marginBottom: SP.gap.md }}>
                <Dot color={severityColor(sev)} size={6} />
                <Text style={{ fontFamily: F.bold, fontSize: 8.5, color: C.secondary, textTransform: "capitalize" }}>
                  {sev}  ·  {items.length} issue{items.length > 1 ? "s" : ""}
                </Text>
              </View>
              {items.map((dp) => <PatternCard key={dp.pattern.id} dp={dp} />)}
              <View style={{ ...s.hr, marginBottom: SP.gap.xl }} />
            </View>
          );
        })}

        {data.detectedPatterns.length === 0 && (
          <View style={{ alignItems: "center", paddingVertical: SP.gap.xxl }}>
            <Text style={{ fontFamily: F.bold, fontSize: 14, color: C.success }}>No issues detected</Text>
            <Text style={{ ...s.caption, marginTop: SP.gap.sm }}>Your site is in excellent health.</Text>
          </View>
        )}
      </View>
      <PageFooter domain={domain} />
    </Page>
  );
}

// ── Pillar + Summary Page ─────────────────────────────────────────────────────

function PillarBar({ label, score }: { label: string; score: number }) {
  const fill  = Math.max(0, Math.min(100, score));
  const color = pillarColor(label.toLowerCase());

  return (
    <View style={{ marginBottom: SP.gap.xl }}>
      <View style={{ ...s.row, justifyContent: "space-between", marginBottom: 4 }}>
        <View style={{ ...s.row, gap: SP.gap.sm }}>
          <Dot color={color} size={5} />
          <Text style={{ fontFamily: F.bold, fontSize: 9, color: C.text }}>{label}</Text>
        </View>
        <Text style={{ fontFamily: F.bold, fontSize: 9, color: scoreColor(score) }}>
          {score.toFixed(0)}
          <Text style={{ fontFamily: F.regular, color: C.faint }}>/100</Text>
        </Text>
      </View>
      {/* Track */}
      <View style={{ height: 4, backgroundColor: C.border, borderRadius: 2 }}>
        {/* Fill */}
        <View style={{ height: 4, width: `${fill}%` as unknown as number, backgroundColor: color, borderRadius: 2, opacity: 0.75 }} />
      </View>
    </View>
  );
}

function SummaryPage({ data }: { data: ReportData }) {
  const domain = data.url.replace(/^https?:\/\//, "").replace(/\/$/, "");
  const { pillarScores } = data;

  return (
    <Page size="A4" style={s.page}>
      <PageBar left={`${domain} · Summary`} right="SEO Insikt" />
      <View style={s.body}>

        {/* Pillar bars */}
        <Text style={s.h2}>Pillar Health</Text>
        <View style={{ marginTop: SP.gap.xl }}>
          {[
            ["Technical",     pillarScores.technical],
            ["Content",       pillarScores.content],
            ["Performance",   pillarScores.performance],
            ["Accessibility", pillarScores.accessibility],
          ].map(([label, score]) => (
            <PillarBar key={label as string} label={label as string} score={Math.round(score as number)} />
          ))}
        </View>

        {/* Overall */}
        <View style={{ ...s.row, gap: SP.gap.lg, marginBottom: SP.gap.xxl, marginTop: SP.gap.sm }}>
          <Text style={s.caption}>Overall health score:</Text>
          <Text style={{ fontFamily: F.bold, fontSize: 11, color: scoreColor(pillarScores.overall) }}>
            {pillarScores.overall.toFixed(0)}/100
          </Text>
        </View>

        <View style={s.hr} />

        {/* Issue breakdown */}
        <Section label="Issue Breakdown" mt={SP.gap.xl} />

        <View style={{ flexDirection: "row", gap: SP.gap.xl }}>
          {(["critical", "warning", "suggestion"] as const).map((sev) => {
            const count = data.detectedPatterns.filter((d) => d.pattern.severity === sev).length;
            const col   = severityColor(sev);
            return (
              <View key={sev} style={{ flex: 1 }}>
                <Text style={{ fontFamily: F.mono, fontSize: 22, color: col }}>
                  {count}
                </Text>
                <Text style={{ ...s.caption, marginTop: 2, textTransform: "capitalize" }}>{sev}</Text>
              </View>
            );
          })}
        </View>

        <View style={{ ...s.hr, marginTop: SP.gap.xl }} />

        {/* Per-category breakdown */}
        <Section label="Issues by Pillar" />
        {["technical", "content", "performance", "accessibility"].map((cat) => {
          const items = data.detectedPatterns.filter((d) => d.pattern.category === cat);
          return (
            <View key={cat} style={{ ...s.row, justifyContent: "space-between", marginBottom: SP.gap.md }}>
              <View style={{ ...s.row, gap: SP.gap.sm }}>
                <Dot color={pillarColor(cat)} size={5} />
                <Text style={{ fontSize: 9, color: C.text, textTransform: "capitalize" }}>{cat}</Text>
              </View>
              <Text style={{ fontFamily: F.mono, fontSize: 9, color: items.length > 0 ? C.critical : C.success }}>
                {items.length} issue{items.length !== 1 ? "s" : ""}
              </Text>
            </View>
          );
        })}

        <View style={{ ...s.hr, marginTop: SP.gap.sm }} />

        {/* Site health */}
        <Section label="Site Health" />
        <View style={{ flexDirection: "row", gap: SP.gap.xl }}>
          {[
            { label: "Sitemap",    ok: data.sitemapFound },
            { label: "Robots.txt", ok: data.robotsTxtFound },
          ].map((h) => (
            <View key={h.label} style={{ flex: 1, ...s.row, gap: SP.gap.sm }}>
              <Dot color={h.ok ? C.success : C.critical} size={7} />
              <View>
                <Text style={{ fontFamily: F.bold, fontSize: 9, color: C.text }}>{h.label}</Text>
                <Text style={{ ...s.caption, color: h.ok ? C.success : C.critical }}>
                  {h.ok ? "Found" : "Missing"}
                </Text>
              </View>
            </View>
          ))}
        </View>

      </View>
      <PageFooter domain={domain} />
    </Page>
  );
}

// ── Recommendations Page ──────────────────────────────────────────────────────

function RecommendationsPage({ data }: { data: ReportData }) {
  const domain = data.url.replace(/^https?:\/\//, "").replace(/\/$/, "");
  const items  = data.detectedPatterns.slice(0, 8);

  return (
    <Page size="A4" style={s.page}>
      <PageBar left={`${domain} · Recommendations`} right="SEO Insikt" />
      <View style={s.body}>
        <Text style={s.h2}>Top Recommendations</Text>
        <Text style={{ ...s.caption, marginTop: 4, marginBottom: SP.gap.xl }}>
          Prioritised by severity, business impact, and fix effort.
        </Text>

        {items.map((dp, i) => {
          const col = severityColor(dp.pattern.severity);
          return (
            <View
              key={dp.pattern.id}
              wrap={false}
              style={{
                flexDirection: "row",
                gap: SP.gap.lg,
                paddingVertical: SP.gap.lg,
                borderTopWidth: 0.5,
                borderTopColor: C.border,
              }}
            >
              {/* Index */}
              <View style={{ width: 20, alignItems: "flex-end", flexShrink: 0 }}>
                <Text style={{ fontFamily: F.mono, fontSize: 11, color: C.faint }}>
                  {String(i + 1).padStart(2, "0")}
                </Text>
              </View>

              {/* Content */}
              <View style={{ flex: 1 }}>
                <View style={{ ...s.row, gap: SP.gap.sm, marginBottom: 3 }}>
                  <Dot color={col} size={5} />
                  <Text style={{ fontFamily: F.bold, fontSize: 10, color: C.text }}>
                    {dp.pattern.name}
                  </Text>
                </View>
                <Text style={{ ...s.body2, marginBottom: 4 }}>
                  {dp.pattern.recommendation}
                </Text>
                <View style={{ ...s.row, gap: SP.gap.lg }}>
                  <Text style={s.mono}>{Math.round(dp.prevalence * 100)}% of pages</Text>
                  <Text style={s.mono}>Impact: {dp.pattern.businessImpact}</Text>
                  <Text style={s.mono}>Effort: {dp.pattern.fixEffort}</Text>
                </View>
              </View>
            </View>
          );
        })}

        {items.length === 0 && (
          <Text style={s.body2}>No issues to fix — keep it up.</Text>
        )}
      </View>
      <PageFooter domain={domain} />
    </Page>
  );
}

// ── Narrative Page ────────────────────────────────────────────────────────────

function NarrativePage({ data }: { data: ReportData }) {
  const domain = data.url.replace(/^https?:\/\//, "").replace(/\/$/, "");
  const lines  = data.aiBrief.split("\n");

  return (
    <Page size="A4" style={s.page}>
      <PageBar left={`${domain} · Analysis Narrative`} right="SEO Insikt" />
      <View style={s.body}>
        {lines.map((line, i) => {
          const t = line.trimEnd();
          if (!t) return <View key={i} style={{ marginTop: SP.gap.md }} />;

          // H1 — document title (treated as section break)
          if (t.startsWith("# ")) {
            return (
              <View key={i} wrap={false} style={{ marginTop: SP.gap.xl, marginBottom: SP.gap.md }}>
                <Text style={s.h2}>{stripMd(t.slice(2))}</Text>
                <View style={{ ...s.hr, marginTop: 5 }} />
              </View>
            );
          }
          // H2 — section heading
          if (t.startsWith("## ")) {
            return (
              <View key={i} wrap={false} style={{ marginTop: SP.gap.xl, marginBottom: 5 }}>
                <Text style={{ fontFamily: F.bold, fontSize: 10.5, color: C.text }}>
                  {stripMd(t.slice(3))}
                </Text>
                <View style={{ ...s.hr, marginTop: 4 }} />
              </View>
            );
          }
          // H3 — sub-section heading
          if (t.startsWith("### ")) {
            return (
              <Text key={i} wrap={false} style={{ fontFamily: F.bold, fontSize: 9, color: C.secondary, marginTop: SP.gap.md, marginBottom: 3 }}>
                {stripMd(t.slice(4))}
              </Text>
            );
          }
          // Bullet
          if (t.startsWith("- ") || t.startsWith("* ")) {
            return (
              <View key={i} style={{ flexDirection: "row", gap: SP.gap.sm, marginBottom: 3, paddingLeft: SP.gap.md, alignItems: "flex-start" }}>
                <View style={{ width: 3, height: 3, borderRadius: 1.5, backgroundColor: C.muted, marginTop: 5, flexShrink: 0 }} />
                <Text style={{ ...s.body2, flex: 1 }}>{stripMd(t.slice(2))}</Text>
              </View>
            );
          }
          // Numbered list
          if (/^\d+\.\s/.test(t)) {
            return (
              <View key={i} style={{ flexDirection: "row", gap: SP.gap.sm, marginBottom: 4, paddingLeft: SP.gap.md, alignItems: "flex-start" }}>
                <Text style={{ fontFamily: F.mono, fontSize: 8, color: C.faint, flexShrink: 0, marginTop: 1 }}>
                  {t.match(/^(\d+)\./)?.[1] ?? ""}
                </Text>
                <Text style={{ ...s.body2, flex: 1 }}>{stripMd(t.replace(/^\d+\.\s*/, ""))}</Text>
              </View>
            );
          }
          // Paragraph
          return (
            <Text key={i} style={{ ...s.body1, marginBottom: 5 }}>
              {stripMd(t)}
            </Text>
          );
        })}
      </View>
      <PageFooter domain={domain} />
    </Page>
  );
}

// ── Document root ─────────────────────────────────────────────────────────────

export function SeoReportDocument({ data }: { data: ReportData }) {
  const date = new Date().toLocaleDateString("en-GB", {
    day: "2-digit",
    month: "long",
    year: "numeric",
  });

  return (
    <Document
      title={`SEO Report — ${data.url}`}
      author="SEO Insikt"
      subject="SEO Analysis Report"
    >
      <CoverPage data={data} date={date} />
      <SummaryPage data={data} />
      <FindingsPage data={data} />
      <RecommendationsPage data={data} />
      {data.aiBrief && <NarrativePage data={data} />}
    </Document>
  );
}
