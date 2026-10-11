import { convertFileSrc, isTauri } from "@tauri-apps/api/core";

/** Backend route serving a site's `src/` folder read-only. */
const SITE_FILES_PREFIX = "/site-files";

/** Desktop custom protocol forwarding to that route (`src-tauri/src/site_files.rs`). */
const SITE_FILES_SCHEME = "site-files";

/** Where posts live inside a site's `src/` (Astro content collection). */
const POST_DIR = "content/blog";

/**
 * Base URL of the site-files route: the backend's HTTP route in the browser,
 * the `site-files://` protocol in the Tauri webview. `convertFileSrc` picks
 * the platform's URL shape (`http://site-files.localhost` on Windows).
 */
export function siteFilesBaseUrl(backendUrl: string): string {
  if (!isTauri()) return `${backendUrl}${SITE_FILES_PREFIX}`;
  return convertFileSrc("", SITE_FILES_SCHEME).replace(/\/$/, "");
}

/**
 * Maps an image URL from a post's markdown to one the webview can load.
 * Relative paths are relative to the post file, as Astro resolves them, so
 * they are resolved against the post's location under `baseUrl` (see
 * `siteFilesBaseUrl`). Absolute URLs pass through unchanged.
 */
export function resolveSiteFileUrl(
  baseUrl: string,
  siteSlug: string,
  postId: string,
  url: string,
): string {
  const postFileUrl = `${baseUrl}/${siteSlug}/${POST_DIR}/${postId}.md`;
  return new URL(url, postFileUrl).href;
}
