import { cn } from "@/src/lib/utils";
import { PageAnalysisData } from "@/src/lib/types";
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
  const isBroken = Boolean(
    page.status_code && (page.status_code >= 400 || page.status_code < 200)
  );

  const rowClass = isBroken ? STYLES.broken.row : STYLES.healthy.row;

  return (
    <div
      onClick={() => onClick(index)}
      className={cn(
        "grid px-4 py-3 border-b cursor-pointer items-center",
        GRID_COLS,
        GRID_GAP,
        rowClass
      )}
    >
      <PageInfo url={page.url} title={page.title} isBroken={isBroken} />
      <div className={CELL.base}>
        <LoadTime loadTime={page.load_time} isBroken={isBroken} />
      </div>
      <WordsCell count={page.word_count} isBroken={isBroken} />
      <HeadingCounts h1={page.h1_count} h2={page.h2_count} h3={page.h3_count} isBroken={isBroken} />
      <div className={CELL.base}>
        <ImageCount
          count={page.image_count}
          withoutAlt={page.images_without_alt}
          isBroken={isBroken}
        />
      </div>
      <LinksCell internal={page.internal_links} external={page.external_links} isBroken={isBroken} />
      <div className={CELL.base}>
        {isBroken ? (
          "-"
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
                "text-sm font-medium",
                getScoreColor(page.lighthouse_seo)
              )}
            >
              {page.lighthouse_seo.toPrecision(2)}
            </span>
          ) : (
            <span className="text-muted-foreground">-</span>
          )
        ) : (
          <SeoScore score={page.lighthouse_seo} />
        )}
      </div>

      <ChevronCell />
    </div>
  );
}