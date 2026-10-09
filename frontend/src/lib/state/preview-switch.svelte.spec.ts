import { describe, expect, it } from "vitest";
import { decidePreviewAction } from "./preview-switch.js";
import type { PreviewView } from "#lib/types/bindings.js";

const view = (
  type: PreviewView["state"]["type"],
  slug: string | null,
): PreviewView => ({
  state:
    type === "Failed"
      ? { type: "Failed", payload: { reason: "boom" } }
      : ({ type } as PreviewView["state"]),
  slug,
  url: type === "Running" ? "http://localhost:4321" : null,
});

describe("decidePreviewAction", () => {
  it("returns 'none' when already running for the target site", () => {
    expect(decidePreviewAction(view("Running", "my-blog"), "my-blog")).toBe(
      "none",
    );
  });

  it("returns 'start' when stopped", () => {
    expect(decidePreviewAction(view("Stopped", null), "my-blog")).toBe("start");
  });

  it("returns 'start' when failed", () => {
    expect(decidePreviewAction(view("Failed", "other-blog"), "my-blog")).toBe(
      "start",
    );
  });

  it("returns 'stop-then-start' when running for a different site", () => {
    expect(decidePreviewAction(view("Running", "other-blog"), "my-blog")).toBe(
      "stop-then-start",
    );
  });

  it("returns 'stop-then-start' when starting for a different site", () => {
    expect(decidePreviewAction(view("Starting", "other-blog"), "my-blog")).toBe(
      "stop-then-start",
    );
  });

  it("returns 'none' when already starting for the target site", () => {
    expect(decidePreviewAction(view("Starting", "my-blog"), "my-blog")).toBe(
      "none",
    );
  });

  it("returns 'none' while stopping, regardless of slug", () => {
    expect(decidePreviewAction(view("Stopping", "my-blog"), "my-blog")).toBe(
      "none",
    );
  });
});
