import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";

vi.mock("../job-list/organisms/JobItem", () => ({
  JobItem: ({ job }: { job: { job_id: string; url: string } }) => (
    <div data-testid={`job-${job.job_id}`}>{job.url}</div>
  ),
}));

import { JobList } from "../job-list/JobList";

const makeJob = (id: string, url: string) => ({
  job_id: id,
  url,
  status: "running" as const,
  total_pages: 10,
  analyzed_pages: 5,
  current_url: url,
  started_at: "2026-01-01T00:00:00Z",
});

describe("JobList", () => {
  it("shows empty state when no jobs", () => {
    render(<JobList jobs={[]} onViewResult={vi.fn()} onCancel={vi.fn()} />);
    expect(screen.getByText("No analysis jobs found")).toBeInTheDocument();
  });

  it("shows helper text in empty state", () => {
    render(<JobList jobs={[]} onViewResult={vi.fn()} onCancel={vi.fn()} />);
    expect(screen.getByText(/Submit a URL above/)).toBeInTheDocument();
  });

  it("renders a JobItem for each job", () => {
    const jobs = [makeJob("1", "https://a.com"), makeJob("2", "https://b.com")];
    render(<JobList jobs={jobs} onViewResult={vi.fn()} onCancel={vi.fn()} />);

    expect(screen.getByTestId("job-1")).toBeInTheDocument();
    expect(screen.getByTestId("job-2")).toBeInTheDocument();
  });

  it("does not show empty state when jobs exist", () => {
    render(<JobList jobs={[makeJob("1", "https://a.com")]} onViewResult={vi.fn()} onCancel={vi.fn()} />);
    expect(screen.queryByText("No analysis jobs found")).not.toBeInTheDocument();
  });
});
