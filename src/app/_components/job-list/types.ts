export type ProgressEvent =
  | {
      event: "analysis";
      job_id: string;
      progress: number;
      pages_analyzed: number;
      total_pages: number;
    }
  | {
      event: "discovery";
      job_id: string;
      count: number;
      total_pages: number;
    };
