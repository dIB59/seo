/**
 * SEO Insikt — PDF Report
 *
 * Story arc:
 *   1. Cover           — "what we audited and how bad is it"
 *   2. Where You Stand — "what's the overall picture"
 *   3. What's Hurting You — "which pillars are dragging the site down"
 *   4. Where to Start  — "if I only fix three things, what?"
 *   5. Full Findings   — "what else did you find?"
 *   6. Your Next 30 Days — "what's the plan?"
 *
 * Visual language: Geist Sans / Geist Mono, app blue primary accent,
 * soft rounded surfaces, generous whitespace. Colour reserved for
 * scores, severity, and the brand accent.
 */
import React from "react";
import {
  Document,
  Page,
  View,
  Text,
  Svg,
  Circle,
  Rect,
  Defs,
  LinearGradient,
  Stop,
  StyleSheet,
} from "@react-pdf/renderer";
import type { ReportData, DetectedPattern } from "@/src/bindings";
import {
  C, F, SP,
  scoreColor, scoreGrade, severityColor, pillarColor, stripMd,
} from "./theme";
import { parseBrief, firstSentence, capSentences, type BriefSections } from "./brief";

// ── Base stylesheet ───────────────────────────────────────────────────────────

const s = StyleSheet.create({
  page: {
    backgroundColor: C.pageBg,
    fontFamily: F.regular,
    color: C.text,
    fontSize: 9,
  },

  // Interior page top bar
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

  // Cover page light header (no fill)
  coverHeader: {
    paddingHorizontal: SP.page,
    paddingTop: 22,
    paddingBottom: 14,
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
  },
  wordmark: { fontSize: 9, fontFamily: F.bold, color: C.primary, letterSpacing: 0.4 },
  wordmarkSub: { fontSize: 7, color: C.faint, marginTop: 1, letterSpacing: 0.6, textTransform: "uppercase" },

  body: {
    flex: 1,
    paddingHorizontal: SP.page,
    paddingTop: SP.gap.xl,
    paddingBottom: 44,
  },

  kicker: {
    fontSize: 7,
    fontFamily: F.bold,
    color: C.primaryDim,
    textTransform: "uppercase",
    letterSpacing: 1.2,
    marginBottom: 4,
  },
  sectionLabel: {
    fontSize: 7,
    fontFamily: F.bold,
    color: C.primaryDim,
    textTransform: "uppercase",
    letterSpacing: 1.2,
    marginBottom: 5,
  },

  hr: { borderBottomWidth: 0.5, borderBottomColor: C.border },

  h1:    { fontSize: 22, fontFamily: F.bold, color: C.text, lineHeight: 1.2 },
  h2:    { fontSize: 14, fontFamily: F.bold, color: C.text, lineHeight: 1.25 },
  h3:    { fontSize: 10, fontFamily: F.bold, color: C.secondary, lineHeight: 1.3 },
  body1: { fontSize: 9,   lineHeight: 1.6,  color: C.text },
  body2: { fontSize: 8.5, lineHeight: 1.55, color: C.secondary },
  caption: { fontSize: 7.5, lineHeight: 1.4, color: C.muted },
  mono:    { fontFamily: F.mono, fontSize: 7.5, color: C.muted },

  row:    { flexDirection: "row", alignItems: "center" },
  flex1:  { flex: 1 },
});

// ── Shared primitives ─────────────────────────────────────────────────────────

function Dot({ color, size = 5 }: { color: string; size?: number }) {
  return (
    <View style={{ width: size, height: size, borderRadius: size / 2, backgroundColor: color, flexShrink: 0 }} />
  );
}

function Lede({ children }: { children: React.ReactNode }) {
  return (
    <Text style={{ fontSize: 9.5, lineHeight: 1.5, color: C.secondary, fontStyle: "italic", marginTop: 4, marginBottom: SP.gap.lg }}>
      {children}
    </Text>
  );
}

function Card({
  children,
  alt = false,
  style,
}: {
  children: React.ReactNode;
  alt?: boolean;
  style?: object;
}) {
  return (
    <View
      style={{
        backgroundColor: alt ? C.surfaceAlt : C.surface,
        borderWidth: 0.5,
        borderColor: C.border,
        borderRadius: SP.radius,
        padding: SP.gap.md + 2,
        ...(style ?? {}),
      }}
    >
      {children}
    </View>
  );
}

function PageHeading({ kicker, title, lede }: { kicker: string; title: string; lede?: string }) {
  return (
    <View style={{ marginBottom: SP.gap.md }}>
      <Text style={s.kicker}>{kicker}</Text>
      <Text style={s.h2}>{title}</Text>
      {lede && <Lede>{lede}</Lede>}
    </View>
  );
}

function MetricTile({
  value,
  label,
  color = C.text,
  divider = false,
}: {
  value: string | number;
  label: string;
  color?: string;
  divider?: boolean;
}) {
  return (
    <View
      style={{
        flex: 1,
        paddingRight: SP.gap.xl,
        borderRightWidth: divider ? 0.5 : 0,
        borderRightColor: C.border,
        marginRight: divider ? SP.gap.xl : 0,
      }}
    >
      <Text style={{ fontFamily: F.mono, fontSize: 24, color }}>{value}</Text>
      <Text style={{ ...s.caption, marginTop: 2 }}>{label}</Text>
    </View>
  );
}

/** Score ring — port of the app's ScoreRing SVG. */
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

function PageBar({ left, right }: { left: string; right?: string }) {
  return (
    <View style={s.bar} fixed>
      <Text style={s.barLeft}>{left}</Text>
      {right && <Text style={s.barRight}>{right}</Text>}
    </View>
  );
}

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

/** Cover hero band — soft primary→white SVG gradient behind the score. */
function HeroBand() {
  return (
    <Svg
      width="595" height="200"
      viewBox="0 0 595 200"
      style={{ position: "absolute", top: 0, left: 0, right: 0 }}
    >
      <Defs>
        <LinearGradient id="heroGrad" x1="0" y1="0" x2="0" y2="1">
          <Stop offset="0" stopColor={C.primarySoft} stopOpacity={1} />
          <Stop offset="1" stopColor={C.pageBg} stopOpacity={0} />
        </LinearGradient>
      </Defs>
      <Rect x="0" y="0" width="595" height="200" fill="url(#heroGrad)" />
    </Svg>
  );
}

// ── Cover Page ────────────────────────────────────────────────────────────────

function CoverPage({ data, brief, date }: { data: ReportData; brief: BriefSections; date: string }) {
  const domain = data.url.replace(/^https?:\/\//, "").replace(/\/$/, "");
  const col    = scoreColor(data.seoScore);
  const lede   = firstSentence(brief.diagnosis);

  return (
    <Page size="A4" style={s.page}>
      <HeroBand />

      {/* Light header strip */}
      <View style={s.coverHeader}>
        <View>
          <Text style={s.wordmark}>SEO INSIKT</Text>
          <Text style={s.wordmarkSub}>Analysis Report</Text>
        </View>
        <Text style={{ fontSize: 7, color: C.muted, letterSpacing: 0.4 }}>{date}</Text>
      </View>

      <View style={{ paddingHorizontal: SP.page, paddingTop: SP.gap.xl }}>
        <Text style={{ fontSize: 7, color: C.primaryDim, marginBottom: 6, textTransform: "uppercase", letterSpacing: 1.2, fontFamily: F.bold }}>
          Website
        </Text>
        <Text style={{ fontFamily: F.bold, fontSize: 24, color: C.text, marginBottom: SP.gap.xxl }}>
          {domain}
        </Text>

        {/* Score + grade row */}
        <View style={{ flexDirection: "row", gap: SP.gap.xl, marginBottom: SP.gap.xxl, alignItems: "flex-start" }}>
          <ScoreRing score={data.seoScore} size={96} />
          <View style={{ flex: 1, paddingTop: 4 }}>
            <Text style={{ fontFamily: F.bold, fontSize: 20, color: col, marginBottom: 4 }}>
              {scoreGrade(data.seoScore)}
            </Text>
            <Text style={{ ...s.body2, marginBottom: SP.gap.sm }}>
              {data.totalPages} page{data.totalPages !== 1 ? "s" : ""} analysed
              {" · "}
              {data.totalIssues} issue{data.totalIssues !== 1 ? "s" : ""} detected
            </Text>
            <View style={{ flexDirection: "row", gap: SP.gap.xl, marginTop: 4 }}>
              {[
                { label: "Sitemap",    ok: data.sitemapFound },
                { label: "Robots.txt", ok: data.robotsTxtFound },
              ].map((h) => (
                <View key={h.label} style={{ flexDirection: "row", alignItems: "center", gap: SP.gap.xs }}>
                  <Dot color={h.ok ? C.success : C.critical} size={6} />
                  <Text style={s.caption}>{h.label} {h.ok ? "found" : "missing"}</Text>
                </View>
              ))}
            </View>
          </View>
        </View>

        {/* Pull-quote: first sentence of the AI diagnosis */}
        {lede && (
          <View style={{ marginBottom: SP.gap.xxl, paddingLeft: SP.gap.lg, borderLeftWidth: 2, borderLeftColor: C.primary }}>
            <Text style={{ fontSize: 12, lineHeight: 1.45, color: C.text, fontStyle: "italic" }}>
              {stripMd(lede)}
            </Text>
          </View>
        )}

        <View style={s.hr} />

        {/* Key numbers row */}
        <View style={{ flexDirection: "row", marginTop: SP.gap.xl }}>
          <MetricTile value={data.totalPages}     label="Pages Analysed" divider />
          <MetricTile value={data.totalIssues}    label="Total Issues"   divider />
          <MetricTile value={data.criticalIssues} label="Critical"       color={C.critical} divider />
          <MetricTile value={data.warningIssues}  label="Warnings"       color={C.warning} />
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
                <ScoreRing score={Math.round(n)} size={46} />
                <View style={{ flexDirection: "row", alignItems: "center", gap: SP.gap.xs, marginTop: 6 }}>
                  <Dot color={pillarColor(label as string)} size={4} />
                  <Text style={{ fontSize: 7, color: C.secondary, fontFamily: F.medium }}>{label as string}</Text>
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

// ── What's Hurting You ────────────────────────────────────────────────────────

function PillarBar({ label, score }: { label: string; score: number }) {
  const fill  = Math.max(0, Math.min(100, score));
  const color = pillarColor(label.toLowerCase());

  return (
    <View style={{ marginBottom: SP.gap.lg }}>
      <View style={{ flexDirection: "row", alignItems: "center", justifyContent: "space-between", marginBottom: 4 }}>
        <View style={{ flexDirection: "row", alignItems: "center", gap: SP.gap.sm }}>
          <Dot color={color} size={5} />
          <Text style={{ fontFamily: F.bold, fontSize: 9, color: C.text }}>{label}</Text>
        </View>
        <Text style={{ fontFamily: F.bold, fontSize: 9, color: scoreColor(score) }}>
          {score.toFixed(0)}
          <Text style={{ fontFamily: F.regular, color: C.faint }}>/100</Text>
        </Text>
      </View>
      <View style={{ height: 5, backgroundColor: C.border, borderRadius: 2.5 }}>
        <View style={{ height: 5, width: `${fill}%` as unknown as number, backgroundColor: color, borderRadius: 2.5, opacity: 0.85 }} />
      </View>
    </View>
  );
}

function WhatsHurtingPage({ data, brief }: { data: ReportData; brief: BriefSections }) {
  const domain = data.url.replace(/^https?:\/\//, "").replace(/\/$/, "");
  const { pillarScores } = data;
  const diagnosis = capSentences(brief.diagnosis, 4);

  return (
    <Page size="A4" style={s.page}>
      <PageBar left={`${domain} · 01 What's Hurting You`} right="SEO INSIKT" />
      <View style={s.body}>
        <PageHeading
          kicker="Chapter 1"
          title="What's hurting your visibility"
          lede="The four pillars below reveal where the site is losing ground."
        />

        {/* Diagnosis paragraph — moved here from the deleted Where You Stand page */}
        {diagnosis && (
          <Card style={{ marginTop: SP.gap.sm, marginBottom: SP.gap.xl }}>
            <Text style={{ ...s.body1, color: C.text }}>{stripMd(diagnosis)}</Text>
          </Card>
        )}

        <Text style={s.sectionLabel}>Pillar Health</Text>
        <View style={{ marginTop: SP.gap.md }}>
          {[
            ["Technical",     pillarScores.technical],
            ["Content",       pillarScores.content],
            ["Performance",   pillarScores.performance],
            ["Accessibility", pillarScores.accessibility],
          ].map(([label, score]) => (
            <PillarBar key={label as string} label={label as string} score={Math.round(score as number)} />
          ))}
        </View>

        <View style={{ flexDirection: "row", alignItems: "center", gap: SP.gap.lg, marginBottom: SP.gap.xl }}>
          <Text style={s.caption}>Overall health score:</Text>
          <Text style={{ fontFamily: F.bold, fontSize: 12, color: scoreColor(pillarScores.overall) }}>
            {pillarScores.overall.toFixed(0)}/100
          </Text>
        </View>

        <View style={s.hr} />

        {/* Pattern breakdown — labelled distinctly from the cover's raw issue counts */}
        <Text style={{ ...s.sectionLabel, marginTop: SP.gap.xl }}>Patterns by Severity</Text>
        <View style={{ flexDirection: "row", gap: SP.gap.xl, marginTop: SP.gap.md, marginBottom: SP.gap.xl }}>
          {(["critical", "warning", "suggestion"] as const).map((sev) => {
            const count = data.detectedPatterns.filter((d) => d.pattern.severity === sev).length;
            const col   = severityColor(sev);
            return (
              <View key={sev} style={{ flex: 1 }}>
                <Text style={{ fontFamily: F.medium, fontSize: 22, color: col }}>{count}</Text>
                <Text style={{ ...s.caption, marginTop: 2, textTransform: "capitalize" }}>{sev} patterns</Text>
              </View>
            );
          })}
        </View>

        <Text style={s.sectionLabel}>Patterns by Pillar</Text>
        <View style={{ marginTop: SP.gap.md }}>
          {["technical", "content", "performance", "accessibility"].map((cat) => {
            const items = data.detectedPatterns.filter((d) => d.pattern.category === cat);
            return (
              <View key={cat} style={{ flexDirection: "row", alignItems: "center", justifyContent: "space-between", marginBottom: SP.gap.md }}>
                <View style={{ flexDirection: "row", alignItems: "center", gap: SP.gap.sm }}>
                  <Dot color={pillarColor(cat)} size={5} />
                  <Text style={{ fontSize: 9, color: C.text, textTransform: "capitalize" }}>{cat}</Text>
                </View>
                <Text style={{ fontFamily: F.mono, fontSize: 9, color: items.length > 0 ? C.critical : C.success }}>
                  {items.length} pattern{items.length !== 1 ? "s" : ""}
                </Text>
              </View>
            );
          })}
        </View>
      </View>
      <PageFooter domain={domain} />
    </Page>
  );
}

// ── Where to Start (Priorities) ───────────────────────────────────────────────

function PriorityCard({
  index,
  dp,
  aiNote,
}: {
  index: number;
  dp: DetectedPattern;
  aiNote?: string;
}) {
  const col = severityColor(dp.pattern.severity);
  const pct = Math.round(dp.prevalence * 100);

  return (
    <View
      style={{
        flexDirection: "row",
        gap: SP.gap.lg,
        marginBottom: SP.gap.lg,
      }}
    >
      {/* Numbered marker */}
      <View style={{ width: 36, flexShrink: 0, alignItems: "flex-end", paddingTop: 2 }}>
        <Text style={{ fontFamily: F.monoMed, fontSize: 22, color: C.primary, lineHeight: 1 }}>
          {String(index).padStart(2, "0")}
        </Text>
      </View>

      <View style={{ flex: 1, borderLeftWidth: 2, borderLeftColor: col, paddingLeft: SP.gap.md }}>
        <View style={{ flexDirection: "row", alignItems: "center", justifyContent: "space-between", marginBottom: 3 }}>
          <Text style={{ fontFamily: F.bold, fontSize: 11, color: C.text, flex: 1 }}>
            {dp.pattern.name}
          </Text>
          <Text style={{ fontFamily: F.bold, fontSize: 9, color: col }}>{pct}% of pages</Text>
        </View>

        <Text style={{ ...s.body2, marginBottom: 5 }}>{dp.pattern.description}</Text>

        <View style={{ flexDirection: "row", gap: SP.gap.lg, marginBottom: 6 }}>
          <Text style={s.mono}>Impact: {dp.pattern.businessImpact}</Text>
          <Text style={s.mono}>Effort: {dp.pattern.fixEffort}</Text>
          <Text style={s.mono}>{dp.affectedPages}/{dp.totalPages} pages</Text>
        </View>

        <Card style={{ marginBottom: aiNote ? 6 : 0 }}>
          <Text style={{ fontSize: 7, color: C.primaryDim, marginBottom: 2, textTransform: "uppercase", letterSpacing: 0.6, fontFamily: F.bold }}>
            Recommended fix
          </Text>
          <Text style={{ fontSize: 8.5, color: C.text, lineHeight: 1.45 }}>
            {dp.pattern.recommendation}
          </Text>
        </Card>

        {aiNote && (
          <Card alt>
            <Text style={{ fontSize: 7, color: C.primaryDim, marginBottom: 2, textTransform: "uppercase", letterSpacing: 0.6, fontFamily: F.bold }}>
              Why it matters
            </Text>
            <Text style={{ fontSize: 8.5, color: C.text, lineHeight: 1.45, fontStyle: "italic" }}>
              {stripMd(aiNote)}
            </Text>
          </Card>
        )}
      </View>
    </View>
  );
}

/** Patterns that get the deluxe priority treatment. Findings page hides these. */
const PRIORITY_LIMIT = 3;

function PrioritiesPage({ data, brief }: { data: ReportData; brief: BriefSections }) {
  const domain = data.url.replace(/^https?:\/\//, "").replace(/\/$/, "");
  const items  = data.detectedPatterns.slice(0, PRIORITY_LIMIT);

  return (
    <Page size="A4" style={s.page}>
      <PageBar left={`${domain} · 02 Where to Start`} right="SEO INSIKT" />
      <View style={s.body}>
        <PageHeading
          kicker="Chapter 2"
          title="Where to start Monday morning"
          lede="If you only fix a handful of things, fix these — ranked by impact and effort."
        />

        {items.length === 0 && (
          <View style={{ alignItems: "center", paddingVertical: SP.gap.xxl }}>
            <Text style={{ fontFamily: F.bold, fontSize: 14, color: C.success }}>Nothing urgent to fix.</Text>
            <Text style={{ ...s.caption, marginTop: SP.gap.sm }}>Your site is in excellent health.</Text>
          </View>
        )}

        {items.map((dp, i) => {
          const raw  = brief.priorityByPattern.get(dp.pattern.name.toLowerCase());
          const note = raw ? capSentences(raw, 2) : undefined;
          return <PriorityCard key={dp.pattern.id} index={i + 1} dp={dp} aiNote={note} />;
        })}
      </View>
      <PageFooter domain={domain} />
    </Page>
  );
}

// ── Findings (full list) ──────────────────────────────────────────────────────

function PatternCard({ dp }: { dp: DetectedPattern }) {
  const col = severityColor(dp.pattern.severity);
  const pct = Math.round(dp.prevalence * 100);

  return (
    <View
      wrap={false}
      style={{
        marginBottom: SP.gap.lg,
        borderLeftWidth: 2,
        borderLeftColor: col,
        paddingLeft: SP.gap.md,
      }}
    >
      <View style={{ flexDirection: "row", alignItems: "center", justifyContent: "space-between", marginBottom: 3 }}>
        <Text style={{ fontFamily: F.bold, fontSize: 10, color: C.text, flex: 1 }}>
          {dp.pattern.name}
        </Text>
        <View style={{ flexDirection: "row", alignItems: "center", gap: 4 }}>
          <Text style={{ fontFamily: F.bold, fontSize: 9, color: col }}>{pct}%</Text>
          <Text style={s.caption}>of pages</Text>
        </View>
      </View>

      <Text style={{ ...s.body2, marginBottom: 5 }}>{dp.pattern.description}</Text>

      <View style={{ flexDirection: "row", gap: SP.gap.xl, marginBottom: 5 }}>
        <Text style={s.mono}>{dp.affectedPages}/{dp.totalPages} pages</Text>
        <Text style={s.mono}>Impact: {dp.pattern.businessImpact}</Text>
        <Text style={s.mono}>Effort: {dp.pattern.fixEffort}</Text>
      </View>

      <Card style={{ marginBottom: dp.sampleUrls.length > 0 ? 5 : 0 }}>
        <Text style={{ fontSize: 7, color: C.primaryDim, marginBottom: 2, textTransform: "uppercase", letterSpacing: 0.6, fontFamily: F.bold }}>
          Recommendation
        </Text>
        <Text style={{ fontSize: 8.5, color: C.text, lineHeight: 1.45 }}>
          {dp.pattern.recommendation}
        </Text>
      </Card>

      {dp.sampleUrls.length > 0 && (
        <View style={{ marginTop: 4 }}>
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
  // Skip the patterns that already got the deluxe treatment on the
  // Priorities page — repeating them is the report's biggest source
  // of duplication.
  const remaining = data.detectedPatterns.slice(PRIORITY_LIMIT);
  const grouped: Partial<Record<string, DetectedPattern[]>> = {};
  for (const dp of remaining) {
    (grouped[dp.pattern.severity] ??= []).push(dp);
  }

  return (
    <Page size="A4" style={s.page}>
      <PageBar left={`${domain} · 03 The Rest`} right="SEO INSIKT" />
      <View style={s.body}>
        <PageHeading
          kicker="Chapter 3"
          title="The rest of the findings"
          lede={`Everything else we detected beyond the top ${PRIORITY_LIMIT} priorities.`}
        />

        {["critical", "warning", "suggestion"].map((sev) => {
          const items = grouped[sev] ?? [];
          if (items.length === 0) return null;
          return (
            <View key={sev}>
              <View style={{ flexDirection: "row", alignItems: "center", gap: SP.gap.sm, marginBottom: SP.gap.md, marginTop: SP.gap.md }}>
                <Dot color={severityColor(sev)} size={6} />
                <Text style={{ fontFamily: F.bold, fontSize: 9, color: C.secondary }}>
                  <Text style={{ textTransform: "capitalize" }}>{sev}</Text>
                  {"  ·  "}
                  {items.length} pattern{items.length > 1 ? "s" : ""}
                </Text>
              </View>
              {items.map((dp) => <PatternCard key={dp.pattern.id} dp={dp} />)}
              <View style={{ ...s.hr, marginBottom: SP.gap.md }} />
            </View>
          );
        })}

        {remaining.length === 0 && (
          <View style={{ alignItems: "center", paddingVertical: SP.gap.xxl }}>
            <Text style={{ fontFamily: F.bold, fontSize: 14, color: C.success }}>
              {data.detectedPatterns.length === 0 ? "No issues detected." : "Nothing else to flag."}
            </Text>
            <Text style={{ ...s.caption, marginTop: SP.gap.sm }}>
              {data.detectedPatterns.length === 0
                ? "Your site is in excellent health."
                : "Everything we found is on the previous page."}
            </Text>
          </View>
        )}
      </View>
      <PageFooter domain={domain} />
    </Page>
  );
}

// ── Your Next 30 Days ─────────────────────────────────────────────────────────

/** Split a markdown blob into displayable paragraph runs, dropping
 *  decorative `---` separators and obvious score-bullet lines. */
function splitParagraphs(md: string): string[] {
  if (!md) return [];
  return md
    .split(/\n\s*\n/)
    .map((p) => p.replace(/^---+$/gm, "").trim())
    .filter((p) => p && !/^[-*]\s*(Technical|Content|Performance|Accessibility)\s*[:\-—]\s*\d+/i.test(p));
}

function NextStepsPage({ brief, domain }: { brief: BriefSections; domain: string }) {
  const paragraphs = splitParagraphs(brief.nextSteps);

  return (
    <Page size="A4" style={s.page}>
      <PageBar left={`${domain} · 04 Your Next 30 Days`} right="SEO INSIKT" />
      <View style={s.body}>
        <PageHeading
          kicker="Chapter 4"
          title="Your next 30 days"
          lede="A short, sequenced plan to turn this audit into measurable gains."
        />

        <Text style={{ ...s.sectionLabel, marginTop: SP.gap.md }}>Roadmap</Text>
        <Card>
          {paragraphs.length > 0 ? (
            paragraphs.map((p, i) => (
              <Text
                key={i}
                style={{ ...s.body1, color: C.text, marginBottom: i < paragraphs.length - 1 ? SP.gap.md : 0 }}
              >
                {stripMd(p)}
              </Text>
            ))
          ) : (
            <Text style={s.body2}>
              Address critical issues first, then warnings in order of page coverage.
              Re-audit after each sprint to track movement.
            </Text>
          )}
        </Card>

        {/* CTA bar */}
        <View
          style={{
            marginTop: SP.gap.xxl,
            backgroundColor: C.primarySoft,
            borderRadius: SP.radius,
            borderLeftWidth: 3,
            borderLeftColor: C.primary,
            padding: SP.gap.lg,
          }}
        >
          <Text style={{ fontSize: 8, color: C.primaryDim, fontFamily: F.bold, textTransform: "uppercase", letterSpacing: 1, marginBottom: 4 }}>
            Next Action
          </Text>
          <Text style={{ fontSize: 11, color: C.text, fontFamily: F.bold, marginBottom: 3 }}>
            Re-audit after each fix sprint to track movement.
          </Text>
          <Text style={s.body2}>
            Run SEO Insikt again after applying the priority fixes — the score
            difference is the cleanest signal that the work is paying off.
          </Text>
        </View>
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
  const brief = parseBrief(data.aiBrief);

  return (
    <Document
      title={`SEO Report — ${data.url}`}
      author="SEO Insikt"
      subject="SEO Analysis Report"
    >
      <CoverPage        data={data} brief={brief} date={date} />
      <WhatsHurtingPage data={data} brief={brief} />
      <PrioritiesPage   data={data} brief={brief} />
      <FindingsPage     data={data} />
      <NextStepsPage    brief={brief} domain={data.url.replace(/^https?:\/\//, "").replace(/\/$/, "")} />
    </Document>
  );
}
