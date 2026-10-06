//! Drives an ACP coding agent (Claude Code via `claude-agent-acp`) to draft
//! blog posts inside a site folder.
//!
//! Spike: the agent writes the post file itself; the site's file watcher picks
//! it up and broadcasts `PostChanged`, so no extra wire events are needed yet.
use crate::astro::posts::CONTENT_DIR;
use agent_client_protocol::schema::ProtocolVersion;
use agent_client_protocol::schema::v1::{
    ContentBlock, InitializeRequest, NewSessionRequest, PermissionOptionKind, PromptRequest,
    RequestPermissionOutcome, RequestPermissionRequest, RequestPermissionResponse,
    SelectedPermissionOutcome, SessionNotification, SessionUpdate, StopReason, TextContent,
    ToolKind,
};
use agent_client_protocol::{AcpAgent, Agent, Client, ConnectionTo};
use std::path::{Component, Path};
use tracing::{debug, info};

/// Claude Code's ACP adapter, pinned so an adapter release can't change
/// behaviour under us. Runs the user's installed, signed-in Claude Code.
/// Must be older than npm's `minimum-release-age` (7 days here) or `npx`
/// fails with `ETARGET`.
const AGENT_COMMAND: [&str; 3] = ["npx", "-y", "@agentclientprotocol/claude-agent-acp@0.81.2"];

/// Asks the agent to write `{site_dir}/src/content/blog/{id}.md` about `topic`
/// and waits for its turn to end.
///
/// # Errors
///
/// The site dir can't be resolved, the agent process fails to start, or any
/// ACP request fails (including the agent not being signed in).
pub async fn draft_post(
    site_dir: &Path,
    id: &str,
    topic: &str,
) -> Result<StopReason, agent_client_protocol::Error> {
    // The agent reports absolute paths; a relative `SITES_DIR` would make
    // every location fail the policy's `starts_with` check.
    let site_dir = site_dir
        .canonicalize()
        .map_err(agent_client_protocol::Error::into_internal_error)?;
    let blog_dir = site_dir.join(CONTENT_DIR);
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let prompt = draft_prompt(id, topic, &today);
    let agent = AcpAgent::from_args(AGENT_COMMAND)?;

    Client
        .builder()
        .name("blog-engine")
        .on_receive_notification(
            async move |notification: SessionNotification, _cx| {
                log_update(&notification.update);
                Ok(())
            },
            agent_client_protocol::on_receive_notification!(),
        )
        .on_receive_request(
            async move |request: RequestPermissionRequest, responder, _cx| {
                let outcome = decide_permission(&request, &blog_dir);
                info!(tool = ?request.tool_call.fields.title, ?outcome, "agent permission");
                responder.respond(RequestPermissionResponse::new(outcome))
            },
            agent_client_protocol::on_receive_request!(),
        )
        .connect_with(agent, async move |cx: ConnectionTo<Agent>| {
            cx.send_request(InitializeRequest::new(ProtocolVersion::V1))
                .block_task()
                .await?;
            let session = cx
                .send_request(NewSessionRequest::new(site_dir))
                .block_task()
                .await?;
            let response = cx
                .send_request(PromptRequest::new(
                    session.session_id,
                    vec![ContentBlock::Text(TextContent::new(prompt))],
                ))
                .block_task()
                .await?;
            Ok(response.stop_reason)
        })
        .await
}

fn draft_prompt(id: &str, topic: &str, today: &str) -> String {
    format!(
        "Write a draft blog post about: {topic}\n\n\
         Create exactly one new file: {CONTENT_DIR}/{id}.md. Start it with YAML \
         frontmatter containing `title`, `description` and `pubDate: {today}`, \
         matching the existing posts in that folder. Do not change any other file \
         and do not run shell commands."
    )
}

/// Spike visibility: agent output goes to the backend log until the UI shows it.
fn log_update(update: &SessionUpdate) {
    match update {
        SessionUpdate::AgentMessageChunk(chunk) => {
            if let ContentBlock::Text(text) = &chunk.content {
                debug!(text = %text.text, "agent says");
            }
        }
        SessionUpdate::ToolCall(call) => info!(title = %call.title, "agent tool call"),
        _ => {}
    }
}

/// Answers an agent's permission request on the owner's behalf.
///
/// Non-technical owners can't judge prompts like "allow `pnpm add`?", so the
/// policy is fixed: only edits to `.md` files inside `blog_dir` are allowed;
/// everything else (shell, network, files elsewhere) is rejected.
#[must_use]
pub fn decide_permission(
    request: &RequestPermissionRequest,
    blog_dir: &Path,
) -> RequestPermissionOutcome {
    let fields = &request.tool_call.fields;
    let locations = fields.locations.as_deref().unwrap_or_default();
    let allowed = fields.kind == Some(ToolKind::Edit)
        && !locations.is_empty()
        && locations.iter().all(|l| is_post_file(&l.path, blog_dir));

    let wanted = if allowed {
        PermissionOptionKind::AllowOnce
    } else {
        PermissionOptionKind::RejectOnce
    };

    // No matching option means the agent offered no way to say what we want;
    // cancelling is the only answer that never grants anything.
    request
        .options
        .iter()
        .find(|o| o.kind == wanted)
        .map_or(RequestPermissionOutcome::Cancelled, |o| {
            RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(o.option_id.clone()))
        })
}

/// `starts_with` compares whole components but does not resolve `..`, so a
/// path with any parent-dir component is refused outright.
fn is_post_file(path: &Path, blog_dir: &Path) -> bool {
    path.starts_with(blog_dir)
        && !path.components().any(|c| c == Component::ParentDir)
        && path.extension().is_some_and(|e| e == "md")
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_client_protocol::schema::v1::{
        PermissionOption, ToolCallLocation, ToolCallUpdate, ToolCallUpdateFields,
    };

    const BLOG_DIR: &str = "/sites/my-blog/src/content/blog";

    fn request(kind: ToolKind, paths: &[&str]) -> RequestPermissionRequest {
        let locations = paths
            .iter()
            .map(|p| ToolCallLocation::new(*p))
            .collect::<Vec<_>>();
        let fields = ToolCallUpdateFields::new().kind(kind).locations(locations);
        RequestPermissionRequest::new(
            "session-1",
            ToolCallUpdate::new("call-1", fields),
            vec![
                PermissionOption::new("allow", "Allow", PermissionOptionKind::AllowOnce),
                PermissionOption::new("reject", "Reject", PermissionOptionKind::RejectOnce),
            ],
        )
    }

    fn selected(id: &'static str) -> RequestPermissionOutcome {
        RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(id))
    }

    #[test]
    fn allows_editing_markdown_inside_blog_dir() {
        let req = request(
            ToolKind::Edit,
            &["/sites/my-blog/src/content/blog/hello.md"],
        );
        assert_eq!(
            decide_permission(&req, Path::new(BLOG_DIR)),
            selected("allow")
        );
    }

    #[test]
    fn rejects_editing_outside_blog_dir() {
        let req = request(ToolKind::Edit, &["/sites/my-blog/astro.config.mjs"]);
        assert_eq!(
            decide_permission(&req, Path::new(BLOG_DIR)),
            selected("reject")
        );
    }

    #[test]
    fn rejects_parent_dir_escape() {
        let req = request(
            ToolKind::Edit,
            &["/sites/my-blog/src/content/blog/../../x.md"],
        );
        assert_eq!(
            decide_permission(&req, Path::new(BLOG_DIR)),
            selected("reject")
        );
    }

    #[test]
    fn rejects_non_markdown_file_in_blog_dir() {
        let req = request(ToolKind::Edit, &["/sites/my-blog/src/content/blog/run.sh"]);
        assert_eq!(
            decide_permission(&req, Path::new(BLOG_DIR)),
            selected("reject")
        );
    }

    #[test]
    fn rejects_shell_commands() {
        let req = request(ToolKind::Execute, &[]);
        assert_eq!(
            decide_permission(&req, Path::new(BLOG_DIR)),
            selected("reject")
        );
    }

    #[test]
    fn rejects_edit_without_locations() {
        let req = request(ToolKind::Edit, &[]);
        assert_eq!(
            decide_permission(&req, Path::new(BLOG_DIR)),
            selected("reject")
        );
    }
}
