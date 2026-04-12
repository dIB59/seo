import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { PersonaSettings } from "../PersonaSettings";

describe("PersonaSettings", () => {
  it("renders the AI Instructions label", () => {
    render(<PersonaSettings persona="" setPersona={vi.fn()} />);
    expect(screen.getByText("AI Instructions")).toBeInTheDocument();
  });

  it("displays the persona text in the textarea", () => {
    render(<PersonaSettings persona="Be concise and direct" setPersona={vi.fn()} />);
    const textarea = screen.getByRole("textbox");
    expect(textarea).toHaveValue("Be concise and direct");
  });

  it("calls setPersona on input", async () => {
    const setPersona = vi.fn();
    const user = userEvent.setup();
    render(<PersonaSettings persona="" setPersona={setPersona} />);

    await user.type(screen.getByRole("textbox"), "Hi");
    expect(setPersona).toHaveBeenCalled();
  });

  it("shows the Markdown Supported hint", () => {
    render(<PersonaSettings persona="" setPersona={vi.fn()} />);
    expect(screen.getByText("Markdown Supported")).toBeInTheDocument();
  });

  it("shows the scope description", () => {
    render(<PersonaSettings persona="" setPersona={vi.fn()} />);
    expect(
      screen.getByText(/Applies to all AI models/),
    ).toBeInTheDocument();
  });
});
