import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

vi.mock("swr", () => ({
  default: vi.fn(),
}));

vi.mock("@/src/api/extension", () => ({
  listCustomExtractors: vi.fn(),
  createCustomExtractor: vi.fn(),
  updateCustomExtractor: vi.fn(),
  deleteCustomExtractor: vi.fn(),
}));

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

vi.mock("../ExtractorDialog", () => ({
  ExtractorDialog: ({ open }: { open: boolean }) =>
    open ? <div data-testid="extractor-dialog">Dialog</div> : null,
}));

vi.mock("../ExtractorRow", () => ({
  ExtractorRow: ({ extractor }: { extractor: { id: string; name: string } }) => (
    <tr data-testid={`ext-row-${extractor.id}`}>
      <td>{extractor.name}</td>
    </tr>
  ),
}));

import useSWR from "swr";
import { ExtractorsSettings } from "../ExtractorsSettings";

const mockedUseSWR = vi.mocked(useSWR);

beforeEach(() => vi.clearAllMocks());

describe("ExtractorsSettings", () => {
  it("shows empty state when no extractors", () => {
    mockedUseSWR.mockReturnValue({
      data: [],
      mutate: vi.fn(),
      isLoading: false,
      isValidating: false,
      error: undefined,
    } as never);

    render(<ExtractorsSettings />);
    expect(screen.getByText("No extractors configured.")).toBeInTheDocument();
  });

  it("renders extractor rows", () => {
    mockedUseSWR.mockReturnValue({
      data: [
        { id: "e1", name: "OG Image", tag: "og_image", selector: "meta", attribute: "content", multiple: false, enabled: true },
      ],
      mutate: vi.fn(),
      isLoading: false,
      isValidating: false,
      error: undefined,
    } as never);

    render(<ExtractorsSettings />);
    expect(screen.getByTestId("ext-row-e1")).toBeInTheDocument();
    expect(screen.getByText("OG Image")).toBeInTheDocument();
  });

  it("opens dialog on Add Extractor click", async () => {
    mockedUseSWR.mockReturnValue({
      data: [],
      mutate: vi.fn(),
      isLoading: false,
      isValidating: false,
      error: undefined,
    } as never);

    const user = userEvent.setup();
    render(<ExtractorsSettings />);

    await user.click(screen.getByRole("button", { name: /Add Extractor/i }));
    expect(screen.getByTestId("extractor-dialog")).toBeInTheDocument();
  });
});
