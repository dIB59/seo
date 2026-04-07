import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

vi.mock("../SelectorLivePreview", () => ({
  SelectorLivePreview: () => <div data-testid="live-preview" />,
}));

import { ExtractorDialog } from "../ExtractorDialog";
import type { CustomExtractor } from "@/src/api/extension";

const sampleExtractor: CustomExtractor = {
  id: "id-1",
  name: "OG Image",
  key: "og_image",
  selector: "meta[property='og:image']",
  attribute: "content",
  multiple: false,
  enabled: true,
};

describe("ExtractorDialog", () => {
  beforeEach(() => vi.clearAllMocks());

  it("calls onSave with the typed form values when creating", async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    const onValidationError = vi.fn();
    render(
      <ExtractorDialog
        open
        editing={null}
        saving={false}
        onOpenChange={() => {}}
        onSave={onSave}
        onValidationError={onValidationError}
      />,
    );

    await user.type(screen.getByLabelText(/^Name$/), "Title");
    await user.type(screen.getByLabelText(/^Key/), "title");
    await user.type(screen.getByLabelText(/CSS Selector/), "title");

    await user.click(screen.getByRole("button", { name: /Create/ }));

    expect(onValidationError).not.toHaveBeenCalled();
    expect(onSave).toHaveBeenCalledTimes(1);
    expect(onSave.mock.calls[0][0]).toMatchObject({
      name: "Title",
      key: "title",
      selector: "title",
      enabled: true,
      multiple: false,
    });
  });

  it("reports a validation error when required fields are empty", async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    const onValidationError = vi.fn();
    render(
      <ExtractorDialog
        open
        editing={null}
        saving={false}
        onOpenChange={() => {}}
        onSave={onSave}
        onValidationError={onValidationError}
      />,
    );

    await user.click(screen.getByRole("button", { name: /Create/ }));

    expect(onSave).not.toHaveBeenCalled();
    expect(onValidationError).toHaveBeenCalledWith(
      "Name, key, and selector are required",
    );
  });

  it("pre-fills the form when editing an existing extractor", () => {
    render(
      <ExtractorDialog
        open
        editing={sampleExtractor}
        saving={false}
        onOpenChange={() => {}}
        onSave={() => {}}
        onValidationError={() => {}}
      />,
    );

    expect(screen.getByLabelText(/^Name$/)).toHaveValue("OG Image");
    expect(screen.getByLabelText(/^Key/)).toHaveValue("og_image");
    expect(screen.getByLabelText(/CSS Selector/)).toHaveValue(
      "meta[property='og:image']",
    );
    expect(screen.getByRole("button", { name: /Save Changes/ })).toBeInTheDocument();
  });
});
