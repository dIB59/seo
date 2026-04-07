import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { ReportPatternRow } from "../ReportPatternRow";
import { Table, TableBody } from "@/src/components/ui/table";
import type { ReportPattern } from "@/src/api/report";

const custom: ReportPattern = {
  id: "p-1",
  name: "Missing OG Images",
  description: "",
  category: "content",
  severity: "warning",
  field: "extracted:og_image",
  operator: "missing",
  threshold: null,
  minPrevalence: 0.1,
  businessImpact: "medium",
  fixEffort: "low",
  recommendation: "Add og:image",
  enabled: true,
  isBuiltin: false,
};

const builtin: ReportPattern = { ...custom, id: "p-2", isBuiltin: true, name: "Built-in rule" };

function renderRow(pattern: ReportPattern) {
  const props = {
    pattern,
    onEdit: vi.fn(),
    onDelete: vi.fn(),
    onToggle: vi.fn(),
  };
  render(
    <Table>
      <TableBody>
        <ReportPatternRow {...props} />
      </TableBody>
    </Table>,
  );
  return props;
}

describe("ReportPatternRow", () => {
  it("renders custom pattern with edit and delete buttons", async () => {
    const user = userEvent.setup();
    const props = renderRow(custom);

    expect(screen.getByText("Missing OG Images")).toBeInTheDocument();
    expect(screen.getByText("extracted:og_image")).toBeInTheDocument();
    expect(screen.getByText("is missing")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /Edit pattern/ }));
    expect(props.onEdit).toHaveBeenCalledWith(custom);

    await user.click(screen.getByRole("button", { name: /Delete pattern/ }));
    expect(props.onDelete).toHaveBeenCalledWith("p-1");
  });

  it("hides edit and delete buttons for built-in patterns", () => {
    renderRow(builtin);
    expect(screen.queryByRole("button", { name: /Edit pattern/ })).toBeNull();
    expect(screen.queryByRole("button", { name: /Delete pattern/ })).toBeNull();
  });
});
