use crate::error::AgentError;
use crate::state::AppState;
use admin_protocol::{Command, Envelope, ErrorCode, Event};
use backon::{ExponentialBuilder, Retryable};
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use http_body_util::Empty;
use hyper::ext::Protocol;
use hyper::{Method, Request, Version};
use hyper_util::rt::{TokioExecutor, TokioIo};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::Role;
use uuid::Uuid;

pub async fn agent_main_loop(backend_url: String, state: Arc<AppState>) {
    let run = || async { connect_and_run(backend_url.clone(), state.clone()).await };

    run.retry(
        ExponentialBuilder::default()
            .with_jitter()
            .with_max_delay(Duration::from_mins(1))
            .with_max_times(0),
    )
    .sleep(tokio::time::sleep)
    .notify(|err: &anyhow::Error, dur: Duration| {
        tracing::error!(error = %err, delay_secs = dur.as_secs(), "reconnecting to backend");
    })
    .await
    .ok();
}

pub async fn connect_and_run(url: String, state: Arc<AppState>) -> anyhow::Result<()> {
    // Step 1: parse "ws://host:port/path" into a TCP address and an HTTP/2 URI.
    // h2 CONNECT uses http scheme, not ws — axum's extractor matches on path only.
    let uri = url.parse::<hyper::Uri>()?;
    let host = uri.host().ok_or_else(|| anyhow::anyhow!("missing host in url: {url}"))?;
    let port = uri.port_u16().unwrap_or(80);
    let addr = format!("{host}:{port}");
    let path = uri.path_and_query().map_or("/", |pq| pq.as_str());
    let http_uri: hyper::Uri = format!("http://{addr}{path}").parse()?;

    // Step 2: TCP connect.
    let tcp = TcpStream::connect(&addr).await?;

    // Step 3: HTTP/2 connection handshake — produces SendRequest + Connection.
    // Connection drives the h2 framing loop (flow control, PING, window updates).
    // It must be spawned concurrently; without it SendRequest deadlocks on backpressure.
    let (mut sender, conn) =
        hyper::client::conn::http2::handshake(TokioExecutor::new(), TokioIo::new(tcp)).await?;
    tokio::spawn(conn);

    // Step 4: Extended CONNECT request per RFC 8441.
    // :protocol is a pseudo-header — it must go through the extensions map, not headers.
    let req = Request::builder()
        .method(Method::CONNECT)
        .uri(http_uri)
        .version(Version::HTTP_2)
        .extension(Protocol::from_static("websocket"))
        .header("sec-websocket-version", "13")
        .body(Empty::<Bytes>::new())?;

    // Step 5: Send the request. Expect 200 OK — h2 never sends 101.
    let res = sender.send_request(req).await?;
    if res.status() != hyper::StatusCode::OK {
        anyhow::bail!("server rejected WS upgrade: {}", res.status());
    }

    // Step 6: Resolve the raw bidirectional h2 stream.
    let upgraded = hyper::upgrade::on(res).await?;

    // Step 7: Wrap in WebSocketStream without running the WS handshake again —
    // the Extended CONNECT exchange was the handshake. Role::Client enables frame masking.
    let ws_stream =
        WebSocketStream::from_raw_socket(TokioIo::new(upgraded), Role::Client, None).await;

    tracing::info!("connected to backend via h2c");

    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        let msg = msg?;
        let Message::Text(text) = msg else { continue };
        let envelope: Envelope<Command> = serde_json::from_str(&text)?;
        tracing::debug!(command = ?envelope.payload, id = %envelope.id, "received command");

        let event = dispatch_command(envelope.payload, &state).await;
        tracing::debug!(event = ?event, "dispatched, sending reply");

        let reply = Envelope {
            id: Uuid::new_v4(),
            correlation_id: Some(envelope.id),
            idempotency_key: None,
            sequence: 0,
            timestamp: chrono::Utc::now(),
            payload: event,
        };
        let json = serde_json::to_string(&reply)?;
        write.send(Message::Text(json.into())).await?;
    }

    tracing::info!("disconnected from backend cleanly");
    Ok(())
}

async fn dispatch_command(cmd: Command, state: &Arc<AppState>) -> Event {
    match cmd {
        Command::Ping => Event::Pong,
        Command::CreateSite { name, slug } => {
            match crate::handlers::sites::create_site(&state.sites_dir, &name, &slug) {
                Err(AgentError::SiteAlreadyExists(_)) => Event::Error {
                    code: ErrorCode::Conflict,
                    message: format!("site '{slug}' already exists"),
                    command_id: None,
                },
                Err(e) => Event::Error {
                    code: ErrorCode::Internal,
                    message: format!("{e}"),
                    command_id: None,
                },
                Ok(site) => {
                    let site_dir = state.sites_dir.join(&slug);
                    if let Err(e) = crate::handlers::sites::scaffold_site(&site_dir).await {
                        tracing::error!(slug = %slug, error = %e, "Astro scaffold failed");
                        return Event::Error {
                            code: ErrorCode::Internal,
                            message: format!("scaffold failed: {e}"),
                            command_id: None,
                        };
                    }
                    Event::SiteCreated {
                        site_id: Uuid::new_v4(),
                        name: site.name,
                    }
                }
            }
        }
        Command::StartPreview { slug, port } => {
            let port = port.unwrap_or(state.preview_port);
            match crate::handlers::preview::start_preview(state, &slug, port).await {
                Ok(url) => Event::PreviewReady { slug, url, port },
                Err(AgentError::SiteNotFound(_)) => Event::Error {
                    code: ErrorCode::SiteNotFound,
                    message: format!("site '{slug}' not found"),
                    command_id: None,
                },
                Err(AgentError::PreviewAlreadyRunning(_)) => Event::Error {
                    code: ErrorCode::Conflict,
                    message: "preview already running".to_string(),
                    command_id: None,
                },
                Err(AgentError::DevServerTimeout(_)) => Event::Error {
                    code: ErrorCode::PreviewTimeout,
                    message: format!("dev server for '{slug}' timed out"),
                    command_id: None,
                },
                Err(e) => Event::Error {
                    code: ErrorCode::Internal,
                    message: format!("{e}"),
                    command_id: None,
                },
            }
        }
        Command::StopPreview => match crate::handlers::preview::stop_preview(state).await {
            Ok(()) => Event::PreviewStopped,
            Err(e) => Event::Error {
                code: ErrorCode::Internal,
                message: format!("{e}"),
                command_id: None,
            },
        },
        Command::GetStatus { site_id: _site_id } => Event::BuildProgress {
            build_id: Uuid::new_v4(),
            phase: "not implemented".into(),
            percent: 0.0,
        },
        Command::BuildSite { .. } => Event::Error {
            code: ErrorCode::Internal,
            message: "not implemented".into(),
            command_id: None,
        },
    }
}
