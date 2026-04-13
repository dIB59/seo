import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

// Mock child components to isolate the tab-switching behavior
vi.mock("../ReportPatternsSettings", () => ({
  ReportPatternsSettings: () => <div data-testid="patterns-panel">Patterns</div>,
}));
vi.mock("../ReportTemplateEditor", () => ({
  ReportTemplateEditor: () => <div data-testid="template-panel">Template</div>,
}));
vi.mock("../PersonaSettings", () => ({
  PersonaSettings: ({ persona }: { persona: string }) => (
    <div data-testid="persona-panel">Persona: {persona}</div>
  ),
}));

import { ReportBuilder } from "../ReportBuilder";

describe("ReportBuilder", () => {
  const defaultProps = {
    persona: "Test persona",
    setPersona: vi.fn(),
  };

  it("renders three tabs", () => {
    render(<ReportBuilder {...defaultProps} />);
    expect(screen.getByRole("tab", { name: "Patterns" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Template" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "AI Instructions" })).toBeInTheDocument();
  });

  it("shows Patterns tab by default", () => {
    render(<ReportBuilder {...defaultProps} />);
    expect(screen.getByTestId("patterns-panel")).toBeInTheDocument();
  });

  it("switches to Template tab on click", async () => {
    const user = userEvent.setup();
    render(<ReportBuilder {...defaultProps} />);

    await user.click(screen.getByRole("tab", { name: "Template" }));
    expect(screen.getByTestId("template-panel")).toBeInTheDocument();
  });

  it("switches to AI Instructions tab and shows persona", async () => {
    const user = userEvent.setup();
    render(<ReportBuilder {...defaultProps} />);

    await user.click(screen.getByRole("tab", { name: "AI Instructions" }));
    expect(screen.getByTestId("persona-panel")).toBeInTheDocument();
    expect(screen.getByText("Persona: Test persona")).toBeInTheDocument();
  });
});
