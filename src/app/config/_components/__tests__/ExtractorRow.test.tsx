import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { ExtractorRow } from "../ExtractorRow";
import { Table, TableBody } from "@/src/components/ui/table";
import type { CustomExtractor } from "@/src/api/extension";

const extractor: CustomExtractor = {
  id: "id-1",
  name: "Canonical",
  key: "canonical",
  selector: "link[rel='canonical']",
  attribute: "href",
  multiple: false,
  enabled: true,
};

function renderRow(overrides: Partial<Parameters<typeof ExtractorRow>[0]> = {}) {
  const props = {
    extractor,
    onEdit: vi.fn(),
    onDelete: vi.fn(),
    onToggleEnabled: vi.fn(),
    ...overrides,
  };
  render(
    <Table>
      <TableBody>
        <ExtractorRow {...props} />
      </TableBody>
    </Table>,
  );
  return props;
}

describe("ExtractorRow", () => {
  it("renders extractor name, key, and selector", () => {
    renderRow();
    expect(screen.getByText("Canonical")).toBeInTheDocument();
    expect(screen.getByText("canonical")).toBeInTheDocument();
    expect(screen.getByText("link[rel='canonical']")).toBeInTheDocument();
    expect(screen.getByText("@href")).toBeInTheDocument();
  });

  it("calls onEdit when the edit button is clicked", async () => {
    const user = userEvent.setup();
    const props = renderRow();
    await user.click(screen.getByRole("button", { name: /Edit extractor/ }));
    expect(props.onEdit).toHaveBeenCalledWith(extractor);
  });

  it("calls onDelete with the id when the delete button is clicked", async () => {
    const user = userEvent.setup();
    const props = renderRow();
    await user.click(screen.getByRole("button", { name: /Delete extractor/ }));
    expect(props.onDelete).toHaveBeenCalledWith("id-1");
  });
});
