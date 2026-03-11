import { cn } from "@/src/lib/utils";
import type { PageAnalysisData } from "@/src/api/analysis";
import { getScoreColor } from "@/src/lib/seo-metrics";

interface PageRowProps {
  page: PageAnalysisData;
  index: number;
  onClick: (index: number) => void;
}

import {
  GRID_COLS,
  GRID_GAP,
  CELL,
  STYLES,
  StatusIcons,
  SeoScore,
  PageInfo,
  LoadTime,
  ImageCount,
  ChevronCell,
  HeadingCounts,
  WordsCell,
  LinksCell,
} from "../atoms/PageRowAtoms";

export function PageRow({ page, index, onClick }: PageRowProps) {
  const isBroken = Boolean(page.status_code && (page.status_code >= 400 || page.status_code < 200));

  const rowClass = isBroken ? STYLES.broken.row : STYLES.healthy.row;

  return (
    <div
      onClick={() => onClick(index)}
      role="button"
      tabIndex={0}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onClick(index);
        }
      }}
      className={cn(
        "grid px-4 py-2.5 border-b border-border/20 cursor-pointer items-center transition-all duration-150",
        GRID_COLS,
        GRID_GAP,
        rowClass,
      )}
    >
      <PageInfo
        url={page.url}
        title={page.title}
        isBroken={isBroken}
        statusCode={page.status_code}
      />
      <div className={CELL.base}>
        <LoadTime loadTime={page.load_time} isBroken={isBroken} />
      </div>
      <WordsCell count={page.word_count} isBroken={isBroken} />
      <HeadingCounts
        h1={page.headings.filter((h) => h.tag === "h1").length}
        h2={page.headings.filter((h) => h.tag === "h2").length}
        h3={page.headings.filter((h) => h.tag === "h3").length}
        isBroken={isBroken}
      />
      <div className={CELL.base}>
        <ImageCount
          count={page.image_count}
          withoutAlt={page.images_without_alt}
          isBroken={isBroken}
        />
      </div>
      <LinksCell
        internal={page.internal_links}
        external={page.external_links}
        isBroken={isBroken}
      />
      <div className={CELL.base}>
        {isBroken ? (
          <span className="text-muted-foreground/40">—</span>
        ) : (
          <StatusIcons
            mobileFriendly={page.mobile_friendly}
            hasStructuredData={page.has_structured_data}
          />
        )}
      </div>

      <div className={CELL.base}>
        {isBroken ? (
          page.lighthouse_seo ? (
            <span
              className={cn(
                "text-xs font-semibold font-mono tabular-nums",
                getScoreColor(page.lighthouse_seo),
              )}
            >
              {page.lighthouse_seo.toPrecision(2)}
            </span>
          ) : (
            <span className="text-muted-foreground/40">—</span>
          )
        ) : (
          <SeoScore score={page.lighthouse_seo} />
        )}
      </div>

      <ChevronCell />
    </div>
  );
}
