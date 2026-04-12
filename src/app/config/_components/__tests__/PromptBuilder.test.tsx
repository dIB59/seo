import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn(), info: vi.fn() },
}));

import { PromptBuilder } from "../PromptBuilder";

const emptyBlocks: never[] = [];
const sampleBlocks = [
  { id: "b1", type: "text" as const, content: "Analyze the following SEO data:" },
  { id: "b2", type: "variable" as const, content: "Score: {score}" },
];

describe("PromptBuilder", () => {
  it("renders existing blocks with type labels", () => {
    render(<PromptBuilder blocks={sampleBlocks} setBlocks={vi.fn()} />);
    expect(screen.getByText("Instruction Text")).toBeInTheDocument();
    expect(screen.getByText("Data Variable")).toBeInTheDocument();
  });

  it("has Add Instruction and Add Data Variable buttons", () => {
    render(<PromptBuilder blocks={emptyBlocks} setBlocks={vi.fn()} />);
    expect(screen.getByRole("button", { name: /Add Instruction/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Add Data Variable/i })).toBeInTheDocument();
  });

  it("calls setBlocks when Add Instruction is clicked", async () => {
    const setBlocks = vi.fn();
    const user = userEvent.setup();
    render(<PromptBuilder blocks={emptyBlocks} setBlocks={setBlocks} />);

    await user.click(screen.getByRole("button", { name: /Add Instruction/i }));
    expect(setBlocks).toHaveBeenCalledTimes(1);
    const newBlocks = setBlocks.mock.calls[0][0];
    expect(newBlocks).toHaveLength(1);
    expect(newBlocks[0].type).toBe("text");
  });

  it("has a Reset button", () => {
    render(<PromptBuilder blocks={sampleBlocks} setBlocks={vi.fn()} />);
    expect(screen.getByRole("button", { name: /Reset/i })).toBeInTheDocument();
  });

  it("renders block content in textareas", () => {
    render(<PromptBuilder blocks={sampleBlocks} setBlocks={vi.fn()} />);
    const textareas = screen.getAllByRole("textbox");
    expect(textareas.length).toBe(2);
  });
});
