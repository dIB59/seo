import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";

const mockPermissions = {
  maxPages: 100,
  isFreeUser: false,
  isLoading: false,
};
vi.mock("@/src/hooks/use-permissions", () => ({
  usePermissions: () => mockPermissions,
}));

vi.mock("../url-input/use-analysis-defaults", () => ({
  useAnalysisDefaults: () => ({
    defaults: {
      max_pages: 50,
      max_depth: 3,
      respect_robots: true,
      include_subdomains: false,
      request_delay_ms: 200,
    },
    isLoading: false,
  }),
}));

// Mock child components to isolate the form shell
vi.mock("../url-input/molecules/UrlInputGroup", () => ({
  UrlInputGroup: ({ url }: { url: string }) => (
    <input data-testid="url-input" defaultValue={url} />
  ),
}));
vi.mock("../url-input/molecules/SettingsCollapsible", () => ({
  AnalysisSettingsCollapsible: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="settings-collapsible">{children}</div>
  ),
}));
vi.mock("../url-input/organisms/AnalysisSettingsFields", () => ({
  AnalysisSettingsFields: () => <div data-testid="settings-fields" />,
}));

import { UrlInputForm } from "../url-input/UrlInputForm";

describe("UrlInputForm", () => {
  it("renders the form when permissions and defaults are loaded", () => {
    render(<UrlInputForm onSubmit={vi.fn()} isLoading={false} />);
    expect(screen.getByTestId("url-input")).toBeInTheDocument();
  });

  it("shows skeleton when permissions are loading", () => {
    mockPermissions.isLoading = true;
    const { container } = render(<UrlInputForm onSubmit={vi.fn()} isLoading={false} />);
    // Skeleton renders, url-input does not
    expect(screen.queryByTestId("url-input")).not.toBeInTheDocument();
    expect(container.querySelector(".animate-pulse, [class*='skeleton']")).toBeTruthy();
    mockPermissions.isLoading = false;
  });

  it("renders settings collapsible", () => {
    render(<UrlInputForm onSubmit={vi.fn()} isLoading={false} />);
    expect(screen.getByTestId("settings-collapsible")).toBeInTheDocument();
  });
});
