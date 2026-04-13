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

import { CustomCheckDialog } from "../CustomCheckDialog";

const defaultProps = {
  open: true,
  editing: null,
  saving: false,
  onOpenChange: vi.fn(),
  onSave: vi.fn(),
  onValidationError: vi.fn(),
};

describe("CustomCheckDialog", () => {
  it("renders New Custom Check title when not editing", () => {
    render(<CustomCheckDialog {...defaultProps} />);
    expect(screen.getByText("New Custom Check")).toBeInTheDocument();
  });

  it("renders Edit Check title when editing", () => {
    const editing = {
      id: "1",
      name: "Test",
      severity: "warning" as const,
      field: "title",
      operator: "missing" as const,
      threshold: null,
      message_template: "Missing title",
      enabled: true,
    };
    render(<CustomCheckDialog {...defaultProps} editing={editing} />);
    expect(screen.getByText("Edit Check")).toBeInTheDocument();
  });

  it("calls onValidationError when name is empty on save", async () => {
    const user = userEvent.setup();
    const onValidationError = vi.fn();
    render(<CustomCheckDialog {...defaultProps} onValidationError={onValidationError} />);

    await user.click(screen.getByRole("button", { name: /create/i }));
    expect(onValidationError).toHaveBeenCalledWith(
      "Name, field, and message template are required",
    );
  });

  it("disables save button when saving is true", () => {
    render(<CustomCheckDialog {...defaultProps} saving={true} />);
    expect(screen.getByRole("button", { name: /create/i })).toBeDisabled();
  });

  it("calls onOpenChange(false) when Cancel is clicked", async () => {
    const user = userEvent.setup();
    const onOpenChange = vi.fn();
    render(<CustomCheckDialog {...defaultProps} onOpenChange={onOpenChange} />);

    await user.click(screen.getByRole("button", { name: /cancel/i }));
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });
});
