import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { CustomCheckRow } from "../CustomCheckRow";
import { Table, TableBody } from "@/src/components/ui/table";
import type { CustomCheck } from "@/src/api/extension";

const check: CustomCheck = {
  id: "c-1",
  name: "Missing Schema",
  severity: "warning",
  field: "schema_types",
  operator: "missing",
  threshold: null,
  message_template: "No schema",
  enabled: true,
};

function renderRow() {
  const props = {
    check,
    onEdit: vi.fn(),
    onDelete: vi.fn(),
    onToggleEnabled: vi.fn(),
  };
  render(
    <Table>
      <TableBody>
        <CustomCheckRow {...props} />
      </TableBody>
    </Table>,
  );
  return props;
}

describe("CustomCheckRow", () => {
  it("renders check name, field, and operator label", () => {
    renderRow();
    expect(screen.getByText("Missing Schema")).toBeInTheDocument();
    expect(screen.getByText("schema_types")).toBeInTheDocument();
    expect(screen.getByText("is missing")).toBeInTheDocument();
  });

  it("invokes edit and delete callbacks", async () => {
    const user = userEvent.setup();
    const props = renderRow();
    await user.click(screen.getByRole("button", { name: /Edit check/ }));
    expect(props.onEdit).toHaveBeenCalledWith(check);
    await user.click(screen.getByRole("button", { name: /Delete check/ }));
    expect(props.onDelete).toHaveBeenCalledWith("c-1");
  });
});
