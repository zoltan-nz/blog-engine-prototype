/**
 * Derives a post id from a title. Mirrors the backend's `validate_id`
 * character class (`^[a-z0-9-]{1,64}$`) so every generated id is guaranteed
 * to pass server-side validation.
 */
export function slugify(title: string): string {
  return title
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 64);
}
