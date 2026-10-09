import type { PreviewView } from "#lib/types/bindings.js";

export type PreviewAction = "none" | "start" | "stop-then-start";

/**
 * Decides what the editing route should do with the preview process on
 * mount or on any `PreviewChanged` event. Pure so it's testable without a
 * socket; the caller (the `/sites/[slug]` page) is responsible for actually
 * sending `StopPreview`/`StartPreview` and re-deciding once the resulting
 * `PreviewChanged` arrives.
 */
export function decidePreviewAction(
  preview: PreviewView,
  targetSlug: string,
): PreviewAction {
  const forTarget = preview.slug === targetSlug;

  switch (preview.state.type) {
    case "Running":
    case "Starting":
      return forTarget ? "none" : "stop-then-start";
    case "Stopping":
      // A stop is already underway; wait for PreviewChanged(Stopped) rather
      // than issuing a second StopPreview.
      return "none";
    case "Stopped":
    case "Failed":
      return "start";
  }
}
