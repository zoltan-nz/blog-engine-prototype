use crate::astro::error::AstroError;
use crate::types::PostMeta;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const FRONTMATTER_DELIMITER_OPEN: &str = "---\n";
const FRONTMATTER_DELIMITER_CLOSE: &str = "\n---\n";

/// Astro content-collection directory for blog posts, relative to a site's root.
pub const CONTENT_DIR: &str = "src/content/blog";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PostFrontMatter {
    pub title: String,
    pub description: String,
    #[serde(rename = "pubDate")]
    pub pub_date: String,
    #[serde(rename = "updatedDate", skip_serializing_if = "Option::is_none")]
    pub updated_date: Option<String>,
    #[serde(rename = "heroImage", skip_serializing_if = "Option::is_none")]
    pub hero_image: Option<String>,
    #[serde(flatten)]
    pub extra: serde_yaml_ng::Mapping,
}

/// Splits raw post file content into parsed frontmatter and the markdown body.
///
/// The body is returned byte-for-byte as it appears after the closing
/// delimiter — no trimming, so `render_post` can reproduce it exactly.
pub fn parse_post(content: &str) -> Result<(PostFrontMatter, String), AstroError> {
    let (front_matter_str, body) = content
        .strip_prefix(FRONTMATTER_DELIMITER_OPEN)
        .and_then(|rest| rest.split_once(FRONTMATTER_DELIMITER_CLOSE))
        .ok_or(AstroError::InvalidPostFormat)?;

    let front_matter: PostFrontMatter = serde_yaml_ng::from_str(front_matter_str)?;

    Ok((front_matter, body.to_string()))
}

/// Renders frontmatter and body back into post file content. Unknown
/// frontmatter keys captured in `extra` are re-emitted by serde's flatten,
/// which is what preserves fields this backend doesn't model (e.g. future
/// page-editor metadata).
pub fn render_post(front_matter: &PostFrontMatter, body: &str) -> String {
    // serde_yaml_ng::to_string never fails for a plain data struct like this
    // (no non-string map keys, no cyclic refs) — an error here would mean a
    // logic bug, not a runtime condition to recover from.
    let front_matter_str =
        serde_yaml_ng::to_string(front_matter).expect("PostFrontMatter always serializes");

    format!("---\n{front_matter_str}---\n{body}")
}

fn to_meta(id: &str, front_matter: &PostFrontMatter) -> PostMeta {
    PostMeta {
        id: id.to_string(),
        title: front_matter.title.clone(),
        description: front_matter.description.clone(),
        pub_date: front_matter.pub_date.clone(),
        updated_date: front_matter.updated_date.clone(),
    }
}

/// Lists post metadata for a site, sorted by `pub_date` descending. Returns
/// an empty list (not an error) when the content directory is missing —
/// sites scaffolded before this feature, or from the `minimal` template,
/// simply have no posts yet.
pub fn list_posts(site_dir: &Path) -> Result<Vec<PostMeta>, AstroError> {
    let content_dir = site_dir.join(CONTENT_DIR);
    if !content_dir.is_dir() {
        return Ok(vec![]);
    }

    let mut posts = Vec::new();
    for entry in fs::read_dir(&content_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let content = fs::read_to_string(&path)?;
        let (front_matter, _) = parse_post(&content)?;
        posts.push(to_meta(id, &front_matter));
    }

    posts.sort_by(|a, b| b.pub_date.cmp(&a.pub_date));
    Ok(posts)
}

/// Reads a single post's metadata and body from disk.
pub fn read_post(site_dir: &Path, id: &str) -> Result<(PostMeta, String), AstroError> {
    let path = site_dir.join(CONTENT_DIR).join(format!("{id}.md"));
    let content =
        fs::read_to_string(&path).map_err(|_| AstroError::PostNotFound(id.to_string()))?;
    let (front_matter, body) = parse_post(&content)?;
    Ok((to_meta(id, &front_matter), body))
}

/// Writes `content` to `path` via write-to-temp-then-rename in the same
/// directory. Astro's dev-server file watcher polls this directory on every
/// autosave; a plain `fs::write` would let it observe a half-written file
/// mid-write, since `fs::write` is not atomic. `fs::rename` within one
/// filesystem is.
fn write_atomic(path: &Path, content: &str) -> std::io::Result<()> {
    let tmp_path = path.with_extension("md.tmp");
    fs::write(&tmp_path, content)?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}

fn post_path(site_dir: &Path, id: &str) -> std::path::PathBuf {
    site_dir.join(CONTENT_DIR).join(format!("{id}.md"))
}

/// Creates a new post file. Errors `PostAlreadyExists` if `id` is taken;
/// creates the content directory if this is the site's first post.
pub fn create_post(
    site_dir: &Path,
    id: &str,
    title: &str,
    description: &str,
    body: &str,
    pub_date: &str,
) -> Result<PostMeta, AstroError> {
    let path = post_path(site_dir, id);
    if path.exists() {
        return Err(AstroError::PostAlreadyExists(id.to_string()));
    }

    fs::create_dir_all(site_dir.join(CONTENT_DIR))?;

    let front_matter = PostFrontMatter {
        title: title.to_string(),
        description: description.to_string(),
        pub_date: pub_date.to_string(),
        updated_date: None,
        hero_image: None,
        extra: serde_yaml_ng::Mapping::new(),
    };
    write_atomic(&path, &render_post(&front_matter, body))?;

    Ok(to_meta(id, &front_matter))
}

/// Rewrites an existing post's title, description, and body, stamping
/// `updated_date`. Reads the existing file first so `hero_image` and any
/// unmodeled frontmatter keys survive untouched.
pub fn update_post(
    site_dir: &Path,
    id: &str,
    title: &str,
    description: &str,
    body: &str,
    updated_date: &str,
) -> Result<PostMeta, AstroError> {
    let path = post_path(site_dir, id);
    let existing =
        fs::read_to_string(&path).map_err(|_| AstroError::PostNotFound(id.to_string()))?;
    let (mut front_matter, _) = parse_post(&existing)?;

    front_matter.title = title.to_string();
    front_matter.description = description.to_string();
    front_matter.updated_date = Some(updated_date.to_string());

    write_atomic(&path, &render_post(&front_matter, body))?;

    Ok(to_meta(id, &front_matter))
}

/// Deletes a post file. Errors `PostNotFound` if it does not exist.
pub fn delete_post(site_dir: &Path, id: &str) -> Result<(), AstroError> {
    let path = post_path(site_dir, id);
    fs::remove_file(&path).map_err(|_| AstroError::PostNotFound(id.to_string()))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::TempDir;

    fn write_post(site_dir: &std::path::Path, id: &str, content: &str) {
        fs::create_dir_all(site_dir.join(super::CONTENT_DIR)).unwrap();
        fs::write(
            site_dir.join(super::CONTENT_DIR).join(format!("{id}.md")),
            content,
        )
        .unwrap();
    }

    #[test]
    fn list_posts_returns_empty_for_missing_content_dir() {
        let site = TempDir::new().unwrap();

        let result = super::list_posts(site.path()).unwrap();

        assert_eq!(result, vec![]);
    }

    #[test]
    fn list_posts_scans_md_files_sorted_by_pub_date_desc() {
        let site = TempDir::new().unwrap();
        write_post(
            site.path(),
            "older-post",
            "---\ntitle: Older\ndescription: d\npubDate: 2026-01-01\n---\nbody\n",
        );
        write_post(
            site.path(),
            "newer-post",
            "---\ntitle: Newer\ndescription: d\npubDate: 2026-06-01\n---\nbody\n",
        );

        let result = super::list_posts(site.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, "newer-post");
        assert_eq!(result[1].id, "older-post");
    }

    #[test]
    fn list_posts_ignores_non_md_files() {
        let site = TempDir::new().unwrap();
        write_post(
            site.path(),
            "real-post",
            "---\ntitle: Real\ndescription: d\npubDate: 2026-01-01\n---\nbody\n",
        );
        fs::write(
            site.path().join(super::CONTENT_DIR).join("sample.mdx"),
            "not scanned",
        )
        .unwrap();
        fs::write(
            site.path().join(super::CONTENT_DIR).join("notes.txt"),
            "not scanned",
        )
        .unwrap();

        let result = super::list_posts(site.path()).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "real-post");
    }

    #[test]
    fn read_post_returns_meta_and_body() {
        let site = TempDir::new().unwrap();
        write_post(
            site.path(),
            "my-post",
            "---\ntitle: My Post\ndescription: d\npubDate: 2026-01-01\n---\nHello.\n",
        );

        let (meta, body) = super::read_post(site.path(), "my-post").unwrap();

        assert_eq!(meta.id, "my-post");
        assert_eq!(meta.title, "My Post");
        assert_eq!(body, "Hello.\n");
    }

    #[test]
    fn read_post_missing_file_is_post_not_found() {
        let site = TempDir::new().unwrap();

        let result = super::read_post(site.path(), "ghost");

        assert!(matches!(
            result,
            Err(crate::astro::error::AstroError::PostNotFound(id)) if id == "ghost"
        ));
    }

    #[test]
    fn create_post_writes_file_and_returns_meta() {
        let site = TempDir::new().unwrap();

        let meta = super::create_post(
            site.path(),
            "my-post",
            "My Post",
            "desc",
            "body\n",
            "2026-01-01",
        )
        .unwrap();

        assert_eq!(meta.id, "my-post");
        assert_eq!(meta.title, "My Post");
        assert_eq!(meta.pub_date, "2026-01-01");
        let content =
            fs::read_to_string(site.path().join(super::CONTENT_DIR).join("my-post.md")).unwrap();
        assert!(content.contains("title: My Post"));
        assert!(content.contains("body\n"));
    }

    #[test]
    fn create_post_rejects_existing_id() {
        let site = TempDir::new().unwrap();
        super::create_post(
            site.path(),
            "my-post",
            "My Post",
            "desc",
            "body",
            "2026-01-01",
        )
        .unwrap();

        let result = super::create_post(
            site.path(),
            "my-post",
            "Again",
            "desc",
            "body",
            "2026-01-01",
        );

        assert!(matches!(
            result,
            Err(crate::astro::error::AstroError::PostAlreadyExists(id)) if id == "my-post"
        ));
    }

    #[test]
    fn create_post_creates_content_dir_when_missing() {
        let site = TempDir::new().unwrap();

        super::create_post(
            site.path(),
            "my-post",
            "My Post",
            "desc",
            "body",
            "2026-01-01",
        )
        .unwrap();

        assert!(site.path().join(super::CONTENT_DIR).is_dir());
    }

    #[test]
    fn update_post_rewrites_and_stamps_updated_date() {
        let site = TempDir::new().unwrap();
        super::create_post(
            site.path(),
            "my-post",
            "Old Title",
            "old desc",
            "old body",
            "2026-01-01",
        )
        .unwrap();

        let meta = super::update_post(
            site.path(),
            "my-post",
            "New Title",
            "new desc",
            "new body",
            "2026-02-01",
        )
        .unwrap();

        assert_eq!(meta.title, "New Title");
        assert_eq!(meta.updated_date, Some("2026-02-01".to_string()));
        let (_, body) = super::read_post(site.path(), "my-post").unwrap();
        assert_eq!(body, "new body");
    }

    #[test]
    fn update_post_preserves_unknown_frontmatter_keys() {
        let site = TempDir::new().unwrap();
        write_post(
            site.path(),
            "my-post",
            "---\ntitle: Old\ndescription: d\npubDate: 2026-01-01\ncustom: keep-me\n---\nbody\n",
        );

        super::update_post(
            site.path(),
            "my-post",
            "New",
            "d2",
            "new body",
            "2026-02-01",
        )
        .unwrap();

        let content =
            fs::read_to_string(site.path().join(super::CONTENT_DIR).join("my-post.md")).unwrap();
        assert!(content.contains("custom: keep-me"));
    }

    #[test]
    fn update_post_missing_file_is_post_not_found() {
        let site = TempDir::new().unwrap();

        let result = super::update_post(site.path(), "ghost", "New", "d", "body", "2026-02-01");

        assert!(matches!(
            result,
            Err(crate::astro::error::AstroError::PostNotFound(id)) if id == "ghost"
        ));
    }

    #[test]
    fn delete_post_removes_file() {
        let site = TempDir::new().unwrap();
        super::create_post(
            site.path(),
            "my-post",
            "My Post",
            "desc",
            "body",
            "2026-01-01",
        )
        .unwrap();

        super::delete_post(site.path(), "my-post").unwrap();

        assert!(
            !site
                .path()
                .join(super::CONTENT_DIR)
                .join("my-post.md")
                .exists()
        );
    }

    #[test]
    fn delete_post_missing_file_is_post_not_found() {
        let site = TempDir::new().unwrap();

        let result = super::delete_post(site.path(), "ghost");

        assert!(matches!(
            result,
            Err(crate::astro::error::AstroError::PostNotFound(id)) if id == "ghost"
        ));
    }

    #[test]
    fn write_leaves_no_tmp_file_behind() {
        let site = TempDir::new().unwrap();

        super::create_post(
            site.path(),
            "my-post",
            "My Post",
            "desc",
            "body",
            "2026-01-01",
        )
        .unwrap();
        super::update_post(
            site.path(),
            "my-post",
            "Updated",
            "d",
            "body2",
            "2026-02-01",
        )
        .unwrap();

        let entries: Vec<String> = fs::read_dir(site.path().join(super::CONTENT_DIR))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(entries, vec!["my-post.md"]);
    }

    const FIXTURE_ALL_FIELDS: &str = "---\ntitle: Hello World\ndescription: A test post\npubDate: 2026-07-01\nupdatedDate: 2026-07-02\nheroImage: /images/hero.png\n---\nBody text here.\n";

    const FIXTURE_UNKNOWN_KEY: &str = "---\ntitle: Hello World\ndescription: A test post\npubDate: 2026-07-01\ncustom: keep-me\n---\nBody text.\n";

    const FIXTURE_COMPLEX_BODY: &str = "---\ntitle: Hello World\ndescription: A test post\npubDate: 2026-07-01\n---\n# Heading\n\n```rust\nfn main() {}\n```\n\n- one\n- two\n";

    #[test]
    fn parse_post_splits_frontmatter_and_body() {
        let (fm, body) = super::parse_post(FIXTURE_ALL_FIELDS).unwrap();

        assert_eq!(fm.title, "Hello World");
        assert_eq!(fm.description, "A test post");
        assert_eq!(fm.pub_date, "2026-07-01");
        assert_eq!(fm.updated_date, Some("2026-07-02".to_string()));
        assert_eq!(fm.hero_image, Some("/images/hero.png".to_string()));
        assert_eq!(body, "Body text here.\n");
    }

    #[test]
    fn parse_post_rejects_missing_delimiters() {
        let result = super::parse_post("just plain markdown, no frontmatter");

        assert!(matches!(
            result,
            Err(crate::astro::error::AstroError::InvalidPostFormat)
        ));
    }

    #[test]
    fn round_trip_preserves_unknown_keys() {
        let (fm, body) = super::parse_post(FIXTURE_UNKNOWN_KEY).unwrap();

        let rendered = super::render_post(&fm, &body);

        assert!(rendered.contains("custom: keep-me"));
    }

    #[test]
    fn round_trip_preserves_body_exactly() {
        let (fm, body) = super::parse_post(FIXTURE_COMPLEX_BODY).unwrap();

        let rendered = super::render_post(&fm, &body);
        let (_, rendered_body) = super::parse_post(&rendered).unwrap();

        assert_eq!(rendered_body, body);
    }
}
