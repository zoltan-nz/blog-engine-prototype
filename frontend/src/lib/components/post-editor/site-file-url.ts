/** Backend route serving a site's `src/` folder read-only. */
const SITE_FILES_PREFIX = "/site-files";

/** Where posts live inside a site's `src/` (Astro content collection). */
const POST_DIR = "content/blog";

/**
 * Maps an image URL from a post's markdown to one the browser can load.
 * Relative paths are relative to the post file, as Astro resolves them, so
 * they are resolved against the post's location under the backend's
 * site-files route. Absolute URLs pass through unchanged.
 */
export function resolveSiteFileUrl(
  backendUrl: string,
  siteSlug: string,
  postId: string,
  url: string,
): string {
  const postFileUrl = `${backendUrl}${SITE_FILES_PREFIX}/${siteSlug}/${POST_DIR}/${postId}.md`;
  return new URL(url, postFileUrl).href;
}
