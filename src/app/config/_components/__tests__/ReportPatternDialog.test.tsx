import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

vi.mock("@/src/hooks/use-check-field-tags", () => ({
  useCheckFieldTags: () => ({
    tags: [
      { name: "title", label: "Page Title" },
      { name: "tag:og_image", label: "OG Image" },
    ],
    isLoading: false,
  }),
}));

import { ReportPatternDialog } from "../ReportPatternDialog";

const defaultProps = {
  open: true,
  editing: null,
  saving: false,
  onOpenChange: vi.fn(),
  onSave: vi.fn(),
  onValidationError: vi.fn(),
};

describe("ReportPatternDialog", () => {
  it("renders New Report Pattern title when not editing", () => {
    render(<ReportPatternDialog {...defaultProps} />);
    expect(screen.getByText("New Report Pattern")).toBeInTheDocument();
  });

  it("renders Edit Pattern title when editing", () => {
    const editing = {
      id: "1",
      name: "Test Pattern",
      description: "desc",
      category: "technical" as const,
      severity: "warning" as const,
      field: "title",
      operator: "missing" as const,
      threshold: null,
      minPrevalence: 0.1,
      businessImpact: "medium" as const,
      fixEffort: "medium" as const,
      recommendation: "Fix it",
      enabled: true,
    };
    render(<ReportPatternDialog {...defaultProps} editing={editing} />);
    expect(screen.getByText("Edit Pattern")).toBeInTheDocument();
  });

  it("calls onValidationError when required fields are empty", async () => {
    const user = userEvent.setup();
    const onValidationError = vi.fn();
    render(<ReportPatternDialog {...defaultProps} onValidationError={onValidationError} />);

    await user.click(screen.getByRole("button", { name: /create/i }));
    expect(onValidationError).toHaveBeenCalledWith(
      "Name, field, and recommendation are required",
    );
  });

  it("disables save button when saving", () => {
    render(<ReportPatternDialog {...defaultProps} saving={true} />);
    expect(screen.getByRole("button", { name: /create/i })).toBeDisabled();
  });

  it("calls onOpenChange(false) on Cancel click", async () => {
    const user = userEvent.setup();
    const onOpenChange = vi.fn();
    render(<ReportPatternDialog {...defaultProps} onOpenChange={onOpenChange} />);

    await user.click(screen.getByRole("button", { name: /cancel/i }));
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });
});
