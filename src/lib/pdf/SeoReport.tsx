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
import { C, F, S, severityColor, severityBg, pillarColor, scoreColor, scoreLabel } from "./theme";

// ── Base styles ───────────────────────────────────────────────────────────────

const base = StyleSheet.create({
  page: {
    backgroundColor: C.pageBg,
    fontFamily: F.regular,
    color: C.text,
    fontSize: 9,
  },
  body: {
    flex: 1,
    paddingHorizontal: S.margin,
    paddingVertical: S.margin,
  },

  // Text
  label: { fontSize: 7, fontFamily: F.regular, color: C.faint, textTransform: "uppercase", letterSpacing: 0.8 },
  heading1: { fontSize: 18, fontFamily: F.bold, color: C.text },
  heading2: { fontSize: 13, fontFamily: F.bold, color: C.text },
  heading3: { fontSize: 10, fontFamily: F.bold, color: C.text },
  body1: { fontSize: 9, color: C.text, lineHeight: 1.5 },
  body2: { fontSize: 8, color: C.muted, lineHeight: 1.5 },
  mono: { fontFamily: F.mono, fontSize: 8, color: C.muted },

  // Layout
  row: { flexDirection: "row", alignItems: "center" },
  col: { flexDirection: "column" },
  spacer: { marginTop: S.gap.xl },
  spacerSm: { marginTop: S.gap.md },

  // Divider
  divider: { borderBottomWidth: 0.5, borderBottomColor: C.border, marginVertical: S.gap.lg },
  dividerSm: { borderBottomWidth: 0.5, borderBottomColor: C.border, marginVertical: S.gap.md },

  // Card surface
  card: {
    backgroundColor: C.surface,
    borderRadius: 6,
    padding: S.gap.md,
    borderWidth: 0.5,
    borderColor: C.border,
  },
});

// ── Shared primitives ─────────────────────────────────────────────────────────

function Pill({ text, color, bg }: { text: string; color: string; bg: string }) {
  return (
    <View style={{ backgroundColor: bg, borderRadius: 3, paddingHorizontal: 6, paddingVertical: 2 }}>
      <Text style={{ fontSize: 7, fontFamily: F.bold, color, textTransform: "uppercase", letterSpacing: 0.6 }}>
        {text}
      </Text>
    </View>
  );
}

function Dot({ color, size = 6 }: { color: string; size?: number }) {
  return <View style={{ width: size, height: size, borderRadius: size / 2, backgroundColor: color }} />;
}

function MetaChip({ label, value }: { label: string; value: string }) {
  return (
    <View style={{ alignItems: "center", gap: 2 }}>
      <Text style={{ ...base.label }}>{label}</Text>
      <Text style={{ fontFamily: F.bold, fontSize: 13, color: C.text }}>{value}</Text>
    </View>
  );
}

// SVG score ring — mirrors the app's ScoreRing component
function ScoreRing({ score, size = 80 }: { score: number; size?: number }) {
  const sw = 7;
  const r = (size - sw) / 2;
  const cx = size / 2;
  const circ = 2 * Math.PI * r;
  const fill = circ - (circ * Math.max(0, Math.min(100, score))) / 100;
  const color = scoreColor(score);

  return (
    <View style={{ alignItems: "center", justifyContent: "center", position: "relative", width: size, height: size }}>
      <Svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} style={{ position: "absolute" }}>
        {/* Track */}
        <Circle cx={cx} cy={cx} r={r} strokeWidth={sw} stroke={C.border} fill="none" />
        {/* Progress — rotated -90° at centre */}
        <Circle
          cx={cx} cy={cx} r={r}
          strokeWidth={sw} stroke={color} fill="none"
          strokeDasharray={`${circ} ${circ}`}
          strokeDashoffset={fill}
          strokeLinecap="round"
          transform={`rotate(-90 ${cx} ${cx})`}
        />
      </Svg>
      <View style={{ alignItems: "center" }}>
        <Text style={{ fontFamily: F.bold, fontSize: size / 3.2, color, fontFamily: F.mono } as object}>{score}</Text>
        <Text style={{ fontSize: 6.5, color: C.faint }}>/100</Text>
      </View>
    </View>
  );
}

// Thin bar used for pillar scores
function PillarBar({ label, score }: { label: string; score: number }) {
  const color = pillarColor(label.toLowerCase());
  const fill = Math.max(0, Math.min(100, score));

  return (
    <View style={{ marginBottom: S.gap.lg }}>
      <View style={{ ...base.row, justifyContent: "space-between", marginBottom: 4 }}>
        <View style={{ ...base.row, gap: S.gap.sm }}>
          <Dot color={color} size={5} />
          <Text style={{ fontSize: 9, fontFamily: F.bold, color: C.text, textTransform: "capitalize" }}>{label}</Text>
        </View>
        <Text style={{ fontFamily: F.mono, fontSize: 9, color: scoreColor(score) }}>{score.toFixed(0)}</Text>
      </View>
      {/* Track */}
      <View style={{ height: 5, backgroundColor: C.border, borderRadius: 3 }}>
        {/* Fill */}
        <View style={{ height: 5, width: `${fill}%` as unknown as number, backgroundColor: color, borderRadius: 3 }} />
      </View>
    </View>
  );
}

// Page footer (repeats on all interior pages)
function PageFooter({ url }: { url: string }) {
  const domain = url.replace(/^https?:\/\//, "").split("/")[0];
  return (
    <View
      fixed
      style={{
        position: "absolute",
        bottom: 20,
        left: S.margin,
        right: S.margin,
        flexDirection: "row",
        justifyContent: "space-between",
        alignItems: "center",
      }}
    >
      <Text style={{ fontSize: 7, color: C.faint }}>{domain} · SEO Insikt</Text>
      <Text
        style={{ fontSize: 7, color: C.faint }}
        render={({ pageNumber, totalPages }) => `${pageNumber} / ${totalPages}`}
      />
    </View>
  );
}

// Left accent rule on interior pages
function AccentRule() {
  return (
    <View
      fixed
      style={{
        position: "absolute",
        top: 0,
        left: 0,
        bottom: 0,
        width: 3,
        backgroundColor: C.accent,
      }}
    />
  );
}

// Section heading — uppercase label above a hairline
function SectionHead({ label }: { label: string }) {
  return (
    <View style={{ marginBottom: S.gap.lg }}>
      <Text style={{ ...base.label, marginBottom: 4 }}>{label}</Text>
      <View style={{ borderBottomWidth: 0.5, borderBottomColor: C.border }} />
    </View>
  );
}

// ── Cover Page ────────────────────────────────────────────────────────────────

function CoverPage({ data, generatedAt }: { data: ReportData; generatedAt: string }) {
  const sc = scoreColor(data.seoScore);

  return (
    <Page size="A4" style={base.page}>
      {/* Dark header band */}
      <View style={{
        backgroundColor: C.ink,
        paddingHorizontal: S.margin,
        paddingTop: S.marginLg,
        paddingBottom: S.marginLg,
      }}>
        <View style={{ ...base.row, justifyContent: "space-between", alignItems: "flex-start" }}>
          <View style={{ flex: 1 }}>
            <Text style={{ ...base.label, color: C.onDarkMuted, marginBottom: 6 }}>SEO Analysis Report</Text>
            <Text style={{ fontSize: 20, fontFamily: F.bold, color: C.onDark, marginBottom: 4 }}>
              {data.url.replace(/^https?:\/\//, "").replace(/\/$/, "")}
            </Text>
            <Text style={{ fontSize: 8, color: C.onDarkMuted }}>{generatedAt}</Text>
          </View>

          {/* Score ring on dark */}
          <View style={{ alignItems: "center", gap: 4 }}>
            <ScoreRing score={data.seoScore} size={72} />
            <Text style={{ fontSize: 8, fontFamily: F.bold, color: sc, textTransform: "uppercase", letterSpacing: 0.5 }}>
              {scoreLabel(data.seoScore)}
            </Text>
          </View>
        </View>
      </View>

      {/* Stats strip */}
      <View style={{
        flexDirection: "row",
        backgroundColor: C.inkSurface,
        borderBottomWidth: 0.5,
        borderBottomColor: C.inkBorder,
      }}>
        {[
          { label: "Pages Analysed", value: String(data.totalPages) },
          { label: "Total Issues", value: String(data.totalIssues) },
          { label: "Critical", value: String(data.criticalIssues) },
          { label: "Warnings", value: String(data.warningIssues) },
        ].map((s, i) => (
          <View
            key={s.label}
            style={{
              flex: 1,
              paddingVertical: S.gap.md,
              paddingHorizontal: S.gap.md,
              borderLeftWidth: i > 0 ? 0.5 : 0,
              borderLeftColor: C.inkBorder,
              alignItems: "center",
            }}
          >
            <Text style={{ fontFamily: F.bold, fontSize: 16, color: C.onDark, fontFamily: F.mono } as object}>{s.value}</Text>
            <Text style={{ fontSize: 7, color: C.onDarkMuted, marginTop: 2 }}>{s.label}</Text>
          </View>
        ))}
      </View>

      {/* Body */}
      <View style={{ flex: 1, paddingHorizontal: S.margin, paddingTop: S.gap.xxl, paddingBottom: 60 }}>
        {/* Pillar snapshot */}
        <SectionHead label="Pillar Health" />
        <View style={{ flexDirection: "row", gap: S.gap.xl }}>
          {[
            ["Technical", data.pillarScores.technical],
            ["Content", data.pillarScores.content],
            ["Performance", data.pillarScores.performance],
            ["Accessibility", data.pillarScores.accessibility],
          ].map(([label, score]) => {
            const n = score as number;
            const color = pillarColor((label as string).toLowerCase());
            return (
              <View key={label as string} style={{ flex: 1, alignItems: "center", gap: 4 }}>
                <ScoreRing score={Math.round(n)} size={44} />
                <View style={{ ...base.row, gap: 3, alignItems: "center" }}>
                  <Dot color={color} size={4} />
                  <Text style={{ fontSize: 7, color: C.muted, textTransform: "capitalize" }}>{label as string}</Text>
                </View>
              </View>
            );
          })}
        </View>

        <View style={{ ...base.divider, marginTop: S.gap.xxl }} />

        {/* Site health row */}
        <SectionHead label="Site Health" />
        <View style={{ flexDirection: "row", gap: S.gap.xxl }}>
          {[
            { label: "Sitemap", ok: data.sitemapFound },
            { label: "Robots.txt", ok: data.robotsTxtFound },
          ].map((item) => (
            <View key={item.label} style={{ ...base.row, gap: S.gap.sm }}>
              <Dot color={item.ok ? C.success : C.critical} size={7} />
              <Text style={{ fontSize: 9, color: C.text }}>{item.label}</Text>
              <Text style={{ fontSize: 9, color: item.ok ? C.success : C.critical, fontFamily: F.bold }}>
                {item.ok ? "Found" : "Missing"}
              </Text>
            </View>
          ))}
        </View>

        {/* Issue summary row */}
        {data.detectedPatterns.length > 0 && (
          <>
            <View style={{ ...base.divider }} />
            <SectionHead label="Detected Patterns" />
            <View style={{ flexDirection: "row", gap: S.gap.md, flexWrap: "wrap" }}>
              {(["critical", "warning", "suggestion"] as const).map((sev) => {
                const count = data.detectedPatterns.filter((d) => d.pattern.severity === sev).length;
                if (count === 0) return null;
                return (
                  <View key={sev} style={{ ...base.row, gap: S.gap.sm }}>
                    <Dot color={severityColor(sev)} size={6} />
                    <Text style={{ fontSize: 9, color: C.muted }}>
                      <Text style={{ fontFamily: F.bold, color: C.text }}>{count}</Text>
                      {" "}{sev}
                    </Text>
                  </View>
                );
              })}
            </View>
          </>
        )}
      </View>

      {/* Cover footer */}
      <View style={{
        borderTopWidth: 0.5,
        borderTopColor: C.border,
        paddingHorizontal: S.margin,
        paddingVertical: S.gap.md,
        flexDirection: "row",
        justifyContent: "space-between",
        alignItems: "center",
      }}>
        <Text style={{ fontSize: 7, color: C.faint }}>Generated by SEO Insikt</Text>
        <Text style={{ fontSize: 7, color: C.faint }}>{generatedAt}</Text>
      </View>
    </Page>
  );
}

// ── Findings Page ─────────────────────────────────────────────────────────────

function PatternRow({ dp }: { dp: DetectedPattern }) {
  const sev = dp.pattern.severity;
  const color = severityColor(sev);
  const pct = (dp.prevalence * 100).toFixed(0);

  return (
    <View
      wrap={false}
      style={{
        marginBottom: S.gap.lg,
        paddingLeft: S.gap.lg,
        borderLeftWidth: 2,
        borderLeftColor: color,
      }}
    >
      {/* Title row */}
      <View style={{ ...base.row, justifyContent: "space-between", marginBottom: 3 }}>
        <View style={{ ...base.row, gap: S.gap.sm, flex: 1 }}>
          <Text style={{ fontFamily: F.bold, fontSize: 10, color: C.text }}>{dp.pattern.name}</Text>
          <Pill text={sev} color={color} bg={severityBg(sev)} />
        </View>
        <View style={{ ...base.row, gap: S.gap.xs, alignItems: "center" }}>
          <Text style={{ fontFamily: F.mono, fontSize: 9, color, fontFamily: F.bold } as object}>{pct}%</Text>
          <Text style={{ fontSize: 7, color: C.faint }}>of pages</Text>
        </View>
      </View>

      {/* Description */}
      <Text style={{ ...base.body2, marginBottom: 5 }}>{dp.pattern.description}</Text>

      {/* Stats row */}
      <View style={{ ...base.row, gap: S.gap.xl, marginBottom: 5 }}>
        <Text style={base.mono}>{dp.affectedPages}/{dp.totalPages} pages affected</Text>
        <Text style={base.mono}>Impact: {dp.pattern.businessImpact}</Text>
        <Text style={base.mono}>Effort: {dp.pattern.fixEffort}</Text>
        <Text style={base.mono}>Priority: {dp.priorityScore.toFixed(1)}</Text>
      </View>

      {/* Recommendation */}
      <View style={{ ...base.card, padding: S.gap.sm, marginBottom: dp.sampleUrls.length > 0 ? 5 : 0 }}>
        <Text style={{ ...base.label, marginBottom: 2 }}>Recommendation</Text>
        <Text style={{ fontSize: 8.5, color: C.text, lineHeight: 1.4 }}>{dp.pattern.recommendation}</Text>
      </View>

      {/* Sample URLs */}
      {dp.sampleUrls.length > 0 && (
        <View>
          <Text style={{ ...base.label, marginBottom: 2, marginTop: 4 }}>Affected pages (sample)</Text>
          {dp.sampleUrls.slice(0, 3).map((u) => (
            <Text key={u} style={{ fontSize: 7.5, color: C.muted, marginBottom: 1 }}>
              · {u.length > 72 ? u.slice(0, 69) + "…" : u}
            </Text>
          ))}
        </View>
      )}
    </View>
  );
}

function FindingsPage({ data }: { data: ReportData }) {
  const grouped: Partial<Record<string, DetectedPattern[]>> = {};
  for (const dp of data.detectedPatterns) {
    const s = dp.pattern.severity;
    if (!grouped[s]) grouped[s] = [];
    grouped[s]!.push(dp);
  }

  return (
    <Page size="A4" style={base.page}>
      <AccentRule />
      <View style={{ ...base.body, paddingBottom: 48 }}>
        <SectionHead label="Detected Patterns" />

        {["critical", "warning", "suggestion"].map((sev) => {
          const items = grouped[sev] ?? [];
          if (items.length === 0) return null;
          return (
            <View key={sev}>
              <View style={{
                ...base.row,
                gap: S.gap.sm,
                marginBottom: S.gap.md,
                marginTop: sev !== "critical" ? S.gap.xl : 0,
              }}>
                <Dot color={severityColor(sev)} size={6} />
                <Text style={{ fontFamily: F.bold, fontSize: 9, color: C.text, textTransform: "capitalize" }}>
                  {sev} · {items.length}
                </Text>
              </View>
              {items.map((dp) => <PatternRow key={dp.pattern.id} dp={dp} />)}
            </View>
          );
        })}

        {data.detectedPatterns.length === 0 && (
          <View style={{ alignItems: "center", paddingVertical: S.gap.xxl }}>
            <Dot color={C.success} size={10} />
            <Text style={{ fontSize: 11, fontFamily: F.bold, color: C.text, marginTop: S.gap.md }}>
              No patterns detected
            </Text>
            <Text style={{ ...base.body2, marginTop: S.gap.sm }}>Your site is in excellent shape.</Text>
          </View>
        )}
      </View>
      <PageFooter url={data.url} />
    </Page>
  );
}

// ── Recommendations Page ──────────────────────────────────────────────────────

function RecommendationsPage({ data }: { data: ReportData }) {
  const items = data.detectedPatterns.slice(0, 8);

  return (
    <Page size="A4" style={base.page}>
      <AccentRule />
      <View style={{ ...base.body, paddingBottom: 48 }}>
        <SectionHead label="Top Recommendations" />
        <Text style={{ ...base.body2, marginBottom: S.gap.xl }}>
          Prioritised by severity, business impact, and fix effort.
        </Text>

        {items.map((dp, i) => {
          const color = severityColor(dp.pattern.severity);
          return (
            <View
              key={dp.pattern.id}
              wrap={false}
              style={{
                flexDirection: "row",
                gap: S.gap.lg,
                marginBottom: S.gap.lg,
                paddingBottom: S.gap.lg,
                borderBottomWidth: i < items.length - 1 ? 0.5 : 0,
                borderBottomColor: C.border,
              }}
            >
              {/* Number badge */}
              <View style={{
                width: 22,
                height: 22,
                borderRadius: 11,
                borderWidth: 1.5,
                borderColor: color,
                alignItems: "center",
                justifyContent: "center",
                flexShrink: 0,
                marginTop: 1,
              }}>
                <Text style={{ fontFamily: F.bold, fontSize: 9, color }}>{i + 1}</Text>
              </View>

              <View style={{ flex: 1 }}>
                <View style={{ ...base.row, gap: S.gap.sm, marginBottom: 3 }}>
                  <Text style={{ fontFamily: F.bold, fontSize: 10, color: C.text }}>{dp.pattern.name}</Text>
                  <Pill text={dp.pattern.severity} color={color} bg={severityBg(dp.pattern.severity)} />
                </View>
                <Text style={{ fontSize: 9, color: C.text, lineHeight: 1.5, marginBottom: 4 }}>
                  {dp.pattern.recommendation}
                </Text>
                <View style={{ ...base.row, gap: S.gap.lg }}>
                  <Text style={base.mono}>
                    {(dp.prevalence * 100).toFixed(0)}% of pages
                  </Text>
                  <Text style={base.mono}>Impact: {dp.pattern.businessImpact}</Text>
                  <Text style={base.mono}>Effort: {dp.pattern.fixEffort}</Text>
                </View>
              </View>
            </View>
          );
        })}

        {items.length === 0 && (
          <Text style={{ ...base.body2 }}>Nothing to fix — keep it up.</Text>
        )}
      </View>
      <PageFooter url={data.url} />
    </Page>
  );
}

// ── Pillar Detail Page ────────────────────────────────────────────────────────

function PillarPage({ data }: { data: ReportData }) {
  const { pillarScores } = data;
  const pillars: [string, number][] = [
    ["Technical", pillarScores.technical],
    ["Content", pillarScores.content],
    ["Performance", pillarScores.performance],
    ["Accessibility", pillarScores.accessibility],
  ];

  return (
    <Page size="A4" style={base.page}>
      <AccentRule />
      <View style={{ ...base.body, paddingBottom: 48 }}>
        <SectionHead label="Pillar Health" />

        <View style={{ marginBottom: S.gap.xxl }}>
          {pillars.map(([label, score]) => (
            <PillarBar key={label} label={label} score={Math.round(score)} />
          ))}
        </View>

        <View style={base.divider} />

        {/* Overall score + breakdown */}
        <View style={{ flexDirection: "row", gap: S.gap.xxl, alignItems: "flex-start" }}>
          {/* Overall ring */}
          <View style={{ alignItems: "center", gap: S.gap.sm }}>
            <ScoreRing score={Math.round(pillarScores.overall)} size={80} />
            <Text style={{ ...base.label }}>Overall</Text>
          </View>

          {/* Issue breakdown */}
          <View style={{ flex: 1 }}>
            <Text style={{ fontFamily: F.bold, fontSize: 10, color: C.text, marginBottom: S.gap.md }}>
              Issue Breakdown
            </Text>
            {(["critical", "warning", "suggestion"] as const).map((sev) => {
              const count = data.detectedPatterns.filter((d) => d.pattern.severity === sev).length;
              const color = severityColor(sev);
              return (
                <View key={sev} style={{ ...base.row, gap: S.gap.sm, marginBottom: S.gap.md }}>
                  <Dot color={color} size={7} />
                  <Text style={{ fontSize: 9, color: C.text, flex: 1, textTransform: "capitalize" }}>{sev}</Text>
                  <Text style={{ fontFamily: F.bold, fontSize: 11, color, fontFamily: F.mono } as object}>{count}</Text>
                </View>
              );
            })}

            <View style={{ ...base.dividerSm }} />

            <View style={{ ...base.row, gap: S.gap.sm }}>
              <Dot color={data.sitemapFound ? C.success : C.critical} size={6} />
              <Text style={{ fontSize: 8.5, color: C.muted }}>Sitemap {data.sitemapFound ? "found" : "missing"}</Text>
            </View>
            <View style={{ ...base.row, gap: S.gap.sm, marginTop: S.gap.sm }}>
              <Dot color={data.robotsTxtFound ? C.success : C.critical} size={6} />
              <Text style={{ fontSize: 8.5, color: C.muted }}>Robots.txt {data.robotsTxtFound ? "found" : "missing"}</Text>
            </View>
          </View>
        </View>

        {/* Pattern by category */}
        {pillars.map(([cat]) => {
          const catPatterns = data.detectedPatterns.filter(
            (d) => d.pattern.category === cat.toLowerCase(),
          );
          if (catPatterns.length === 0) return null;
          const color = pillarColor(cat.toLowerCase());
          return (
            <View key={cat} style={{ marginTop: S.gap.xl }}>
              <View style={{ ...base.row, gap: S.gap.sm, marginBottom: S.gap.sm }}>
                <Dot color={color} size={5} />
                <Text style={{ fontFamily: F.bold, fontSize: 9, color: C.text }}>{cat}</Text>
                <Text style={{ fontSize: 8, color: C.faint }}>({catPatterns.length})</Text>
              </View>
              {catPatterns.map((dp) => (
                <View key={dp.pattern.id} style={{ ...base.row, gap: S.gap.sm, marginBottom: 4, paddingLeft: S.gap.md }}>
                  <Dot color={severityColor(dp.pattern.severity)} size={4} />
                  <Text style={{ fontSize: 8.5, color: C.text, flex: 1 }}>{dp.pattern.name}</Text>
                  <Text style={{ fontFamily: F.mono, fontSize: 8, color: C.muted }}>
                    {(dp.prevalence * 100).toFixed(0)}%
                  </Text>
                </View>
              ))}
            </View>
          );
        })}
      </View>
      <PageFooter url={data.url} />
    </Page>
  );
}

// ── Narrative Page ────────────────────────────────────────────────────────────

function NarrativePage({ data }: { data: ReportData }) {
  const lines = data.aiBrief.split("\n").filter((l) => l.trim());

  return (
    <Page size="A4" style={base.page}>
      <AccentRule />
      <View style={{ ...base.body, paddingBottom: 48 }}>
        <SectionHead label="Full Narrative" />

        {lines.map((line, i) => {
          const t = line.trim();
          if (t.startsWith("# ")) {
            return (
              <Text key={i} style={{ ...base.heading2, marginTop: S.gap.lg, marginBottom: 4 }}>
                {t.slice(2)}
              </Text>
            );
          }
          if (t.startsWith("## ")) {
            return (
              <Text key={i} style={{ ...base.heading3, marginTop: S.gap.md, marginBottom: 3 }}>
                {t.slice(3)}
              </Text>
            );
          }
          if (t.startsWith("### ")) {
            return (
              <Text key={i} style={{ fontSize: 9, fontFamily: F.bold, color: C.muted, marginTop: S.gap.sm, marginBottom: 2 }}>
                {t.slice(4)}
              </Text>
            );
          }
          if (t.startsWith("- ") || t.startsWith("* ")) {
            return (
              <View key={i} style={{ ...base.row, gap: S.gap.sm, marginBottom: 3, paddingLeft: S.gap.md }}>
                <Dot color={C.accent} size={3} />
                <Text style={{ ...base.body1, flex: 1 }}>{t.slice(2).replace(/\*\*(.+?)\*\*/g, "$1")}</Text>
              </View>
            );
          }
          if (/^\d+\.\s/.test(t)) {
            return (
              <Text key={i} style={{ ...base.body1, paddingLeft: S.gap.md, marginBottom: 3 }}>
                {t.replace(/\*\*(.+?)\*\*/g, "$1")}
              </Text>
            );
          }
          return (
            <Text key={i} style={{ ...base.body1, marginBottom: 4 }}>
              {t.replace(/\*\*(.+?)\*\*/g, "$1")}
            </Text>
          );
        })}
      </View>
      <PageFooter url={data.url} />
    </Page>
  );
}

// ── Root Document ─────────────────────────────────────────────────────────────

export function SeoReportDocument({ data }: { data: ReportData }) {
  const generatedAt = new Date().toLocaleDateString("en-GB", {
    day: "2-digit",
    month: "long",
    year: "numeric",
  });

  return (
    <Document title={`SEO Report — ${data.url}`} author="SEO Insikt" subject="SEO Analysis">
      <CoverPage data={data} generatedAt={generatedAt} />
      <PillarPage data={data} />
      <FindingsPage data={data} />
      <RecommendationsPage data={data} />
      {data.aiBrief && <NarrativePage data={data} />}
    </Document>
  );
}
