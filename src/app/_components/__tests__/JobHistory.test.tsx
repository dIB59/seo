import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";

// Mock Tauri event listener
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

// Mock next/navigation
vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: vi.fn() }),
}));

// Mock API layer
vi.mock("@/src/api/analysis", () => ({
  getPaginatedJobs: vi.fn(),
  cancelAnalysis: vi.fn(),
}));

// Mock SWR to return controlled data
const mockMutate = vi.fn();
vi.mock("swr", () => ({
  default: vi.fn(() => ({
    data: {
      items: [
        {
          job_id: "j1",
          url: "https://example.com",
          status: "completed",
          total_pages: 10,
          analyzed_pages: 10,
          current_url: "https://example.com",
          started_at: "2026-01-01T00:00:00Z",
        },
      ],
      total: 1,
    },
    mutate: mockMutate,
  })),
}));

// Mock child components using the paths as they're imported in JobHistory.tsx
vi.mock("@/src/app/_components/job-list/organisms/JobFilterBar", () => ({
  JobFilterBar: () => <div data-testid="filter-bar" />,
}));
vi.mock("@/src/app/_components/job-list/JobList", () => ({
  JobList: ({ jobs }: { jobs: { job_id: string }[] }) => (
    <div data-testid="job-list">{jobs.length} jobs</div>
  ),
}));
vi.mock("@/src/app/_components/job-list/molecules/JobPagination", () => ({
  JobPagination: () => <div data-testid="pagination" />,
}));

import { JobHistory } from "../job-list/organisms/JobHistory";

beforeEach(() => vi.clearAllMocks());

describe("JobHistory", () => {
  it("renders filter bar, job list, and pagination", () => {
    render(<JobHistory />);
    expect(screen.getByTestId("filter-bar")).toBeInTheDocument();
    expect(screen.getByTestId("job-list")).toBeInTheDocument();
    expect(screen.getByTestId("pagination")).toBeInTheDocument();
  });

  it("passes jobs from SWR to JobList", () => {
    render(<JobHistory />);
    expect(screen.getByText("1 jobs")).toBeInTheDocument();
  });

  it("sets up Tauri event listeners", async () => {
    const { listen } = await import("@tauri-apps/api/event");
    render(<JobHistory />);
    expect(listen).toHaveBeenCalled();
  });
});
