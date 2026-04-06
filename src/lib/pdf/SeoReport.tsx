import React from "react";
import {
  Document,
  Page,
  View,
  Text,
  StyleSheet,
} from "@react-pdf/renderer";
import type { ReportData, DetectedPattern, PillarScores } from "@/src/bindings";
import { COLORS, FONTS, SPACING, severityColor, pillarColor, scoreColor } from "./theme";

// ── Shared styles ─────────────────────────────────────────────────────────────

const styles = StyleSheet.create({
  page: {
    fontFamily: FONTS.regular,
    backgroundColor: COLORS.white,
    paddingTop: SPACING.page,
    paddingBottom: SPACING.page,
    paddingHorizontal: SPACING.page,
    fontSize: 10,
    color: COLORS.gray900,
  },
  // Section headings
  sectionTitle: {
    fontSize: 14,
    fontFamily: FONTS.bold,
    color: COLORS.gray900,
    marginBottom: SPACING.sm,
  },
  subTitle: {
    fontSize: 11,
    fontFamily: FONTS.bold,
    color: COLORS.gray700,
    marginBottom: SPACING.xs,
  },
  // Generic row / column
  row: {
    flexDirection: "row",
    alignItems: "center",
  },
  col: {
    flexDirection: "column",
  },
  spacer: {
    marginTop: SPACING.lg,
  },
  smallSpacer: {
    marginTop: SPACING.sm,
  },
  // Divider
  divider: {
    borderBottomColor: COLORS.gray200,
    borderBottomWidth: 1,
    marginVertical: SPACING.md,
  },
  // Muted text
  muted: {
    color: COLORS.gray500,
    fontSize: 9,
  },
  // Footer
  footer: {
    position: "absolute",
    bottom: SPACING.md,
    left: SPACING.page,
    right: SPACING.page,
    flexDirection: "row",
    justifyContent: "space-between",
    color: COLORS.gray400,
    fontSize: 8,
  },
});

// ── Helpers ───────────────────────────────────────────────────────────────────

function pct(value: number) {
  return `${(value * 100).toFixed(0)}%`;
}

function fmt(n: number) {
  return n.toLocaleString("en-GB");
}

// ── Cover Page ────────────────────────────────────────────────────────────────

function CoverPage({ data, generatedAt }: { data: ReportData; generatedAt: string }) {
  const sc = scoreColor(data.seoScore);

  return (
    <Page size="A4" style={styles.page}>
      {/* Header gradient bar */}
      <View style={{ backgroundColor: COLORS.primaryDark, marginHorizontal: -SPACING.page, marginTop: -SPACING.page, paddingHorizontal: SPACING.page, paddingVertical: SPACING.xxl }}>
        <Text style={{ fontFamily: FONTS.bold, fontSize: 24, color: COLORS.white, marginBottom: SPACING.xs }}>
          SEO Analysis Report
        </Text>
        <Text style={{ fontSize: 11, color: COLORS.primaryLight }}>
          {data.url}
        </Text>
      </View>

      <View style={{ marginTop: SPACING.xxl }} />

      {/* Score card */}
      <View style={{ flexDirection: "row", gap: SPACING.xl, alignItems: "flex-start" }}>
        {/* Big score circle (simulated with a square) */}
        <View style={{
          width: 88,
          height: 88,
          borderRadius: 44,
          backgroundColor: COLORS.gray50,
          borderWidth: 4,
          borderColor: sc,
          alignItems: "center",
          justifyContent: "center",
        }}>
          <Text style={{ fontFamily: FONTS.bold, fontSize: 28, color: sc }}>{data.seoScore}</Text>
          <Text style={{ fontSize: 8, color: COLORS.gray500 }}>/ 100</Text>
        </View>

        <View style={{ flex: 1 }}>
          <Text style={{ fontFamily: FONTS.bold, fontSize: 18, color: COLORS.gray900, marginBottom: SPACING.xs }}>
            {data.seoScore >= 80 ? "Excellent" : data.seoScore >= 50 ? "Needs Attention" : "Critical Issues Found"}
          </Text>
          <Text style={{ fontSize: 10, color: COLORS.gray600, lineHeight: 1.5 }}>
            {data.totalPages} page{data.totalPages !== 1 ? "s" : ""} analysed — {data.totalIssues} issue{data.totalIssues !== 1 ? "s" : ""} detected
          </Text>
          <Text style={{ fontSize: 9, color: COLORS.gray400, marginTop: SPACING.xs }}>
            Generated {generatedAt}
          </Text>
        </View>
      </View>

      <View style={styles.divider} />

      {/* Quick stats */}
      <View style={{ flexDirection: "row", gap: SPACING.md }}>
        {[
          { label: "Critical Issues", value: fmt(data.criticalIssues), color: COLORS.critical },
          { label: "Warnings", value: fmt(data.warningIssues), color: COLORS.warning2 },
          { label: "Pages Analysed", value: fmt(data.totalPages), color: COLORS.info },
          { label: "SEO Score", value: `${data.seoScore}`, color: sc },
        ].map((s) => (
          <View key={s.label} style={{ flex: 1, backgroundColor: COLORS.gray50, borderRadius: 6, padding: SPACING.sm, borderLeftWidth: 3, borderLeftColor: s.color }}>
            <Text style={{ fontFamily: FONTS.bold, fontSize: 18, color: s.color }}>{s.value}</Text>
            <Text style={{ fontSize: 8, color: COLORS.gray500, marginTop: 2 }}>{s.label}</Text>
          </View>
        ))}
      </View>

      <View style={styles.spacer} />

      {/* Site health */}
      <Text style={styles.subTitle}>Site Health</Text>
      <View style={{ flexDirection: "row", gap: SPACING.md }}>
        {[
          { label: "Sitemap", ok: data.sitemapFound },
          { label: "Robots.txt", ok: data.robotsTxtFound },
        ].map((item) => (
          <View key={item.label} style={{ flexDirection: "row", alignItems: "center", gap: SPACING.xs }}>
            <View style={{ width: 8, height: 8, borderRadius: 4, backgroundColor: item.ok ? COLORS.success : COLORS.critical }} />
            <Text style={{ fontSize: 9, color: COLORS.gray600 }}>{item.label}: {item.ok ? "Found" : "Missing"}</Text>
          </View>
        ))}
      </View>

      <Footer url={data.url} pageNumber={1} />
    </Page>
  );
}

// ── Pillar Scores Page ────────────────────────────────────────────────────────

function PillarBar({ label, score }: { label: string; score: number }) {
  const color = pillarColor(label.toLowerCase());
  const fill = Math.max(0, Math.min(100, score));
  const BAR_WIDTH = 400;

  return (
    <View style={{ marginBottom: SPACING.md }}>
      <View style={{ flexDirection: "row", justifyContent: "space-between", marginBottom: 4 }}>
        <Text style={{ fontSize: 10, fontFamily: FONTS.bold, color: COLORS.gray700, textTransform: "capitalize" }}>{label}</Text>
        <Text style={{ fontSize: 10, fontFamily: FONTS.bold, color: scoreColor(score) }}>{score.toFixed(0)}/100</Text>
      </View>
      {/* Track */}
      <View style={{ height: 8, backgroundColor: COLORS.gray200, borderRadius: 4, width: "100%" }}>
        {/* Fill */}
        <View style={{ height: 8, backgroundColor: color, borderRadius: 4, width: `${fill}%` as unknown as number }} />
      </View>
    </View>
  );
}

function SummaryPage({ data, pageNum }: { data: ReportData; pageNum: number }) {
  const { pillarScores } = data;
  const pillars: [string, number][] = [
    ["Technical", pillarScores.technical],
    ["Content", pillarScores.content],
    ["Performance", pillarScores.performance],
    ["Accessibility", pillarScores.accessibility],
  ];

  return (
    <Page size="A4" style={styles.page}>
      <Text style={styles.sectionTitle}>Pillar Health Scores</Text>
      <Text style={{ ...styles.muted, marginBottom: SPACING.lg }}>
        Overall: {pillarScores.overall.toFixed(0)}/100
      </Text>

      {pillars.map(([label, score]) => (
        <PillarBar key={label} label={label} score={score} />
      ))}

      <View style={styles.divider} />

      {/* Pattern summary counts */}
      <Text style={styles.sectionTitle}>Issue Summary</Text>
      {(["critical", "warning", "suggestion"] as const).map((sev) => {
        const count = data.detectedPatterns.filter((d) => d.pattern.severity === sev).length;
        if (count === 0) return null;
        return (
          <View key={sev} style={{ flexDirection: "row", alignItems: "center", gap: SPACING.sm, marginBottom: SPACING.xs }}>
            <View style={{ width: 10, height: 10, borderRadius: 2, backgroundColor: severityColor(sev) }} />
            <Text style={{ fontSize: 10, textTransform: "capitalize" }}>{sev}: </Text>
            <Text style={{ fontSize: 10, fontFamily: FONTS.bold }}>{count} pattern{count !== 1 ? "s" : ""} detected</Text>
          </View>
        );
      })}

      {data.detectedPatterns.length === 0 && (
        <Text style={{ color: COLORS.success, fontSize: 10 }}>No patterns detected — site is in good health.</Text>
      )}

      <Footer url={data.url} pageNumber={pageNum} />
    </Page>
  );
}

// ── Findings Page ─────────────────────────────────────────────────────────────

function PatternCard({ dp }: { dp: DetectedPattern }) {
  const color = severityColor(dp.pattern.severity);
  const affectedPct = pct(dp.prevalence);

  return (
    <View style={{ marginBottom: SPACING.lg, borderLeftWidth: 3, borderLeftColor: color, paddingLeft: SPACING.sm }} wrap={false}>
      {/* Title row */}
      <View style={{ flexDirection: "row", justifyContent: "space-between", alignItems: "flex-start" }}>
        <Text style={{ fontFamily: FONTS.bold, fontSize: 11, color: COLORS.gray900, flex: 1 }}>
          {dp.pattern.name}
        </Text>
        <View style={{ flexDirection: "row", gap: SPACING.xs, alignItems: "center" }}>
          <View style={{ backgroundColor: color, borderRadius: 3, paddingHorizontal: 5, paddingVertical: 2 }}>
            <Text style={{ fontSize: 8, color: COLORS.white, fontFamily: FONTS.bold, textTransform: "uppercase" }}>
              {dp.pattern.severity}
            </Text>
          </View>
          <Text style={{ fontSize: 9, color: COLORS.gray500 }}>{affectedPct} of pages</Text>
        </View>
      </View>

      {/* Description */}
      <Text style={{ fontSize: 9, color: COLORS.gray600, marginTop: 3, lineHeight: 1.4 }}>
        {dp.pattern.description}
      </Text>

      {/* Stats row */}
      <View style={{ flexDirection: "row", gap: SPACING.lg, marginTop: SPACING.xs }}>
        <Text style={styles.muted}>Affected: {fmt(dp.affectedPages)}/{fmt(dp.totalPages)} pages</Text>
        <Text style={styles.muted}>Impact: {dp.pattern.businessImpact}</Text>
        <Text style={styles.muted}>Effort: {dp.pattern.fixEffort}</Text>
        <Text style={styles.muted}>Priority: {dp.priorityScore.toFixed(1)}</Text>
      </View>

      {/* Recommendation */}
      <View style={{ backgroundColor: COLORS.gray50, borderRadius: 4, padding: SPACING.xs, marginTop: SPACING.xs }}>
        <Text style={{ fontSize: 8.5, color: COLORS.gray700, lineHeight: 1.4 }}>
          Fix: {dp.pattern.recommendation}
        </Text>
      </View>

      {/* Sample URLs */}
      {dp.sampleUrls.length > 0 && (
        <View style={{ marginTop: SPACING.xs }}>
          <Text style={{ fontSize: 8, color: COLORS.gray400, marginBottom: 2 }}>Sample affected pages:</Text>
          {dp.sampleUrls.slice(0, 3).map((u) => (
            <Text key={u} style={{ fontSize: 8, color: COLORS.gray500, marginLeft: SPACING.xs }}>
              • {u.length > 70 ? u.slice(0, 67) + "…" : u}
            </Text>
          ))}
        </View>
      )}
    </View>
  );
}

function FindingsPage({ data, pageNum }: { data: ReportData; pageNum: number }) {
  const grouped: Record<string, DetectedPattern[]> = {};
  for (const dp of data.detectedPatterns) {
    const s = dp.pattern.severity;
    if (!grouped[s]) grouped[s] = [];
    grouped[s].push(dp);
  }

  const order = ["critical", "warning", "suggestion"];

  return (
    <Page size="A4" style={styles.page}>
      <Text style={styles.sectionTitle}>Detected Patterns</Text>
      <Text style={{ ...styles.muted, marginBottom: SPACING.md }}>
        Sorted by priority score — highest impact, lowest effort first.
      </Text>

      {order.map((sev) => {
        const items = grouped[sev] ?? [];
        if (items.length === 0) return null;
        return (
          <View key={sev}>
            <View style={{ backgroundColor: severityColor(sev), borderRadius: 4, paddingHorizontal: SPACING.sm, paddingVertical: 4, marginBottom: SPACING.sm }} wrap={false}>
              <Text style={{ fontFamily: FONTS.bold, fontSize: 10, color: COLORS.white, textTransform: "capitalize" }}>
                {sev} ({items.length})
              </Text>
            </View>
            {items.map((dp) => <PatternCard key={dp.pattern.id} dp={dp} />)}
          </View>
        );
      })}

      {data.detectedPatterns.length === 0 && (
        <Text style={{ color: COLORS.success, fontSize: 11, marginTop: SPACING.lg }}>
          No patterns detected. Your site looks great!
        </Text>
      )}

      <Footer url={data.url} pageNumber={pageNum} />
    </Page>
  );
}

// ── Recommendations Page ──────────────────────────────────────────────────────

function RecommendationsPage({ data, pageNum }: { data: ReportData; pageNum: number }) {
  const top = data.detectedPatterns.slice(0, 8);

  return (
    <Page size="A4" style={styles.page}>
      <Text style={styles.sectionTitle}>Top Recommendations</Text>
      <Text style={{ ...styles.muted, marginBottom: SPACING.lg }}>
        Prioritised by impact, severity, and fix effort.
      </Text>

      {top.map((dp, i) => (
        <View key={dp.pattern.id} style={{ flexDirection: "row", gap: SPACING.md, marginBottom: SPACING.md, alignItems: "flex-start" }} wrap={false}>
          {/* Number circle */}
          <View style={{ width: 22, height: 22, borderRadius: 11, backgroundColor: severityColor(dp.pattern.severity), alignItems: "center", justifyContent: "center", marginTop: 1 }}>
            <Text style={{ fontFamily: FONTS.bold, fontSize: 10, color: COLORS.white }}>{i + 1}</Text>
          </View>

          <View style={{ flex: 1 }}>
            <Text style={{ fontFamily: FONTS.bold, fontSize: 10, color: COLORS.gray900, marginBottom: 2 }}>
              {dp.pattern.name}
            </Text>
            <Text style={{ fontSize: 9, color: COLORS.gray600, lineHeight: 1.4 }}>
              {dp.pattern.recommendation}
            </Text>
            <Text style={{ ...styles.muted, marginTop: 2 }}>
              Affects {pct(dp.prevalence)} of pages · {dp.pattern.businessImpact} impact · {dp.pattern.fixEffort} effort
            </Text>
          </View>
        </View>
      ))}

      {top.length === 0 && (
        <Text style={{ color: COLORS.success, fontSize: 11 }}>No issues to fix. Keep it up!</Text>
      )}

      <Footer url={data.url} pageNumber={pageNum} />
    </Page>
  );
}

// ── Narrative Page ────────────────────────────────────────────────────────────

function NarrativePage({ data, pageNum }: { data: ReportData; pageNum: number }) {
  // Render ai_brief line by line with basic Markdown-like parsing
  const lines = data.aiBrief.split("\n").filter((l) => l.trim() !== "");

  return (
    <Page size="A4" style={styles.page}>
      <Text style={styles.sectionTitle}>Full Analysis Narrative</Text>
      <View style={{ ...styles.divider, marginTop: 4 }} />

      {lines.map((line, i) => {
        const trimmed = line.trim();
        if (trimmed.startsWith("# ")) {
          return <Text key={i} style={{ fontFamily: FONTS.bold, fontSize: 13, color: COLORS.primaryDark, marginTop: SPACING.sm, marginBottom: 4 }}>{trimmed.slice(2)}</Text>;
        }
        if (trimmed.startsWith("## ")) {
          return <Text key={i} style={{ fontFamily: FONTS.bold, fontSize: 11, color: COLORS.gray700, marginTop: SPACING.sm, marginBottom: 3 }}>{trimmed.slice(3)}</Text>;
        }
        if (trimmed.startsWith("### ")) {
          return <Text key={i} style={{ fontFamily: FONTS.bold, fontSize: 10, color: COLORS.gray600, marginTop: SPACING.xs, marginBottom: 2 }}>{trimmed.slice(4)}</Text>;
        }
        if (trimmed.startsWith("- ") || trimmed.startsWith("* ")) {
          return <Text key={i} style={{ fontSize: 9, color: COLORS.gray700, marginLeft: SPACING.md, marginBottom: 2, lineHeight: 1.4 }}>• {trimmed.slice(2)}</Text>;
        }
        if (/^\d+\.\s/.test(trimmed)) {
          return <Text key={i} style={{ fontSize: 9, color: COLORS.gray700, marginLeft: SPACING.md, marginBottom: 2, lineHeight: 1.4 }}>{trimmed}</Text>;
        }
        // Bold markers stripped for plain rendering
        const plainLine = trimmed.replace(/\*\*(.+?)\*\*/g, "$1");
        return <Text key={i} style={{ fontSize: 9, color: COLORS.gray700, lineHeight: 1.5, marginBottom: 3 }}>{plainLine}</Text>;
      })}

      <Footer url={data.url} pageNumber={pageNum} />
    </Page>
  );
}

// ── Footer ────────────────────────────────────────────────────────────────────

function Footer({ url, pageNumber }: { url: string; pageNumber: number }) {
  const domain = url.replace(/^https?:\/\//, "").split("/")[0];
  return (
    <View style={styles.footer} fixed>
      <Text>{domain} · SEO Insikt</Text>
      <Text render={({ pageNumber: pn, totalPages }) => `Page ${pn} of ${totalPages}`} />
    </View>
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
    <Document title={`SEO Report — ${data.url}`} author="SEO Insikt">
      <CoverPage data={data} generatedAt={generatedAt} />
      <SummaryPage data={data} pageNum={2} />
      <FindingsPage data={data} pageNum={3} />
      <RecommendationsPage data={data} pageNum={4} />
      {data.aiBrief && <NarrativePage data={data} pageNum={5} />}
    </Document>
  );
}
