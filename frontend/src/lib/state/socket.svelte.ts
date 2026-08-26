import { env } from "$env/dynamic/public";
import type {
  Command,
  ErrorCode,
  Event,
  LogStream,
  PostMeta,
  PreviewView,
  SiteView,
  WsEnvelope,
} from "$lib/types/bindings.js";

export const backendUrl = env.PUBLIC_API_BACKEND_URL || "http://localhost:8080";

const wsUrl = `${backendUrl.replace(/^http/, "ws")}/ws`;

export type ConnectionStatus = "connecting" | "open" | "reconnecting";

export interface BuildLogLine {
  slug: string;
  stream: LogStream;
  data: string;
}

export interface ProtocolError {
  code: ErrorCode;
  message: string;
  correlationId: string | null;
}

const MAX_BUILD_LOG_LINES = 500;
const INITIAL_DELAY_MS = 1_000;
const MAX_DELAY_MS = 30_000;

export const stoppedPreview: PreviewView = {
  state: { type: "Stopped" },
  slug: null,
  url: null,
};

// Pure list reducers, exported for unit tests.
export function upsertSite(sites: SiteView[], site: SiteView): SiteView[] {
  const next = sites.some((s) => s.slug === site.slug)
    ? sites.map((s) => (s.slug === site.slug ? site : s))
    : [...sites, site];
  return next.toSorted((a, b) => a.slug.localeCompare(b.slug));
}

export function removeSite(sites: SiteView[], slug: string): SiteView[] {
  return sites.filter((s) => s.slug !== slug);
}

export function upsertPost(
  posts: Record<string, PostMeta[]>,
  siteSlug: string,
  post: PostMeta,
): Record<string, PostMeta[]> {
  const existing = posts[siteSlug] ?? [];
  const next = existing.some((p) => p.id === post.id)
    ? existing.map((p) => (p.id === post.id ? post : p))
    : [...existing, post];
  next.sort((a, b) => b.pub_date.localeCompare(a.pub_date));
  return { ...posts, [siteSlug]: next };
}

export function removePost(
  posts: Record<string, PostMeta[]>,
  siteSlug: string,
  id: string,
): Record<string, PostMeta[]> {
  const existing = posts[siteSlug];
  if (!existing) return posts;
  return { ...posts, [siteSlug]: existing.filter((p) => p.id !== id) };
}

/**
 * Tracks in-flight `GetPost` requests by correlation id so their `PostBody`/
 * `Error` reply can settle the right promise. Kept independent of the
 * WebSocket connection so it's unit-testable without a live socket.
 */
export class PendingRequests {
  #pending = new Map<
    string,
    { resolve: (body: string) => void; reject: (error: ProtocolError) => void }
  >();

  register(correlationId: string): Promise<string> {
    return new Promise((resolve, reject) => {
      this.#pending.set(correlationId, { resolve, reject });
    });
  }

  settleBody(correlationId: string, body: string): boolean {
    const pending = this.#pending.get(correlationId);
    if (!pending) return false;
    this.#pending.delete(correlationId);
    pending.resolve(body);
    return true;
  }

  settleError(correlationId: string, error: ProtocolError): boolean {
    const pending = this.#pending.get(correlationId);
    if (!pending) return false;
    this.#pending.delete(correlationId);
    pending.reject(error);
    return true;
  }
}

/**
 * Tracks in-flight `UpdatePost` sends. There is no dedicated ack event —
 * success arrives as a `PostChanged` broadcast (matched by site slug + post
 * id, since broadcasts don't carry the request's correlation id) and
 * failure arrives as a correlated `Error`. Whichever settles first wins;
 * the other match key is cleaned up alongside it.
 */
export class PendingUpdates {
  #byCorrelation = new Map<
    string,
    {
      resolve: () => void;
      reject: (error: ProtocolError) => void;
      postKey: string;
    }
  >();
  #byPostKey = new Map<string, string>();

  register(correlationId: string, siteSlug: string, id: string): Promise<void> {
    const postKey = `${siteSlug}:${id}`;
    return new Promise((resolve, reject) => {
      this.#byCorrelation.set(correlationId, { resolve, reject, postKey });
      this.#byPostKey.set(postKey, correlationId);
    });
  }

  settleChanged(siteSlug: string, id: string): boolean {
    const postKey = `${siteSlug}:${id}`;
    const correlationId = this.#byPostKey.get(postKey);
    if (!correlationId) return false;
    const pending = this.#byCorrelation.get(correlationId);
    if (!pending) return false;
    this.#byCorrelation.delete(correlationId);
    this.#byPostKey.delete(postKey);
    pending.resolve();
    return true;
  }

  settleError(correlationId: string, error: ProtocolError): boolean {
    const pending = this.#byCorrelation.get(correlationId);
    if (!pending) return false;
    this.#byCorrelation.delete(correlationId);
    this.#byPostKey.delete(pending.postKey);
    pending.reject(error);
    return true;
  }
}

/**
 * The single WS connection to the backend. State arrives by push: a full
 * `Snapshot` on every (re)connect, then per-entity patches. Commands return
 * their `correlation_id` so callers can match `Error` events back to requests.
 */
export class BlogSocket {
  status = $state<ConnectionStatus>("connecting");
  sites = $state<SiteView[]>([]);
  preview = $state<PreviewView>(stoppedPreview);
  buildLogs = $state<BuildLogLine[]>([]);
  lastError = $state<ProtocolError | null>(null);
  posts = $state<Record<string, PostMeta[]>>({});

  #url: string;
  #socket!: WebSocket;
  #retryDelay = INITIAL_DELAY_MS;
  #retryTimer: ReturnType<typeof setTimeout> | null = null;
  #intentionalClose = false;
  #pending = new PendingRequests();
  #pendingUpdates = new PendingUpdates();

  constructor(url: string) {
    this.#url = url;
    this.#connect();
    window.addEventListener("online", this.#onOnline);
  }

  #connect(): void {
    this.#socket = new WebSocket(this.#url);

    this.#socket.onopen = () => {
      this.status = "open";
      this.#retryDelay = INITIAL_DELAY_MS;
    };

    this.#socket.onclose = () => {
      if (this.#intentionalClose) return;
      this.status = "reconnecting";
      this.#scheduleReconnect();
    };

    this.#socket.onmessage = (event) => {
      try {
        const envelope: WsEnvelope = JSON.parse(event.data);
        if (envelope.type === "Event") {
          if (envelope.payload.type === "PostBody") {
            this.#pending.settleBody(
              envelope.correlation_id,
              envelope.payload.payload.body,
            );
          }
          this.#apply(envelope.payload);
        }
      } catch (err) {
        console.error("Failed to parse WS message", err);
      }
    };

    this.#socket.onerror = (error) => {
      console.error(error);
    };
  }

  #scheduleReconnect(): void {
    this.#retryTimer = setTimeout(() => {
      this.#retryTimer = null;
      this.#connect();
    }, this.#retryDelay);
    this.#retryDelay = Math.min(this.#retryDelay * 2, MAX_DELAY_MS);
  }

  // Arrow field so addEventListener and removeEventListener see the same reference.
  #onOnline = (): void => {
    if (this.#intentionalClose || this.status === "open") return;
    if (this.#retryTimer !== null) {
      clearTimeout(this.#retryTimer);
      this.#retryTimer = null;
    }
    this.#connect();
  };

  #apply(event: Event): void {
    switch (event.type) {
      case "Snapshot":
        this.sites = event.payload.sites;
        this.preview = event.payload.preview;
        this.posts = event.payload.posts;
        break;
      case "SiteChanged":
        this.sites = upsertSite(this.sites, event.payload);
        break;
      case "SiteRemoved":
        this.sites = removeSite(this.sites, event.payload.slug);
        break;
      case "PreviewChanged":
        this.preview = event.payload;
        break;
      case "BuildLog":
        this.buildLogs = [...this.buildLogs, event.payload].slice(
          -MAX_BUILD_LOG_LINES,
        );
        break;
      case "Error": {
        const error: ProtocolError = {
          code: event.payload.code,
          message: event.payload.message,
          correlationId: event.payload.correlation_id,
        };
        this.lastError = error;
        if (event.payload.correlation_id) {
          this.#pending.settleError(event.payload.correlation_id, error);
          this.#pendingUpdates.settleError(event.payload.correlation_id, error);
        }
        break;
      }
      case "Pong":
        break;
      case "PostChanged":
        this.posts = upsertPost(
          this.posts,
          event.payload.site_slug,
          event.payload.post,
        );
        this.#pendingUpdates.settleChanged(
          event.payload.site_slug,
          event.payload.post.id,
        );
        break;
      case "PostRemoved":
        this.posts = removePost(
          this.posts,
          event.payload.site_slug,
          event.payload.id,
        );
        break;
      case "PostBody":
        // Correlation id lives on the envelope, not the payload — settled in
        // `onmessage` before `#apply` sees this event (see `requestPost`).
        break;
    }
  }

  send(command: Command): string {
    const envelope: WsEnvelope = {
      unix_timestamp_us: Date.now() * 1000,
      correlation_id: crypto.randomUUID(),
      type: "Command",
      payload: command,
    };
    if (this.#socket.readyState === WebSocket.OPEN) {
      this.#socket.send(JSON.stringify(envelope));
    }
    return envelope.correlation_id;
  }

  createSite(name: string, slug: string): string {
    return this.send({ type: "CreateSite", payload: { name, slug } });
  }

  buildSite(slug: string): string {
    return this.send({ type: "BuildSite", payload: { slug } });
  }

  startPreview(slug: string): string {
    return this.send({ type: "StartPreview", payload: { slug } });
  }

  stopPreview(): string {
    return this.send({ type: "StopPreview" });
  }

  deleteSite(slug: string): string {
    return this.send({ type: "DeleteSite", payload: { slug } });
  }

  createPost(
    siteSlug: string,
    id: string,
    title: string,
    description: string,
    body: string,
  ): string {
    return this.send({
      type: "CreatePost",
      payload: { site_slug: siteSlug, id, title, description, body },
    });
  }

  deletePost(siteSlug: string, id: string): string {
    return this.send({
      type: "DeletePost",
      payload: { site_slug: siteSlug, id },
    });
  }

  /** Resolves once the matching `PostChanged` broadcast arrives, rejects on
   * a correlated `Error` — see `PendingUpdates`. */
  updatePost(
    siteSlug: string,
    id: string,
    title: string,
    description: string,
    body: string,
  ): Promise<void> {
    const correlationId = this.send({
      type: "UpdatePost",
      payload: { site_slug: siteSlug, id, title, description, body },
    });
    return this.#pendingUpdates.register(correlationId, siteSlug, id);
  }

  /** Resolves with the post's markdown body once the matching `PostBody` (or
   * rejects on a correlated `Error`) arrives. */
  requestPost(siteSlug: string, id: string): Promise<string> {
    const correlationId = this.send({
      type: "GetPost",
      payload: { site_slug: siteSlug, id },
    });
    return this.#pending.register(correlationId);
  }

  ping(): string {
    return this.send({ type: "Ping" });
  }

  dismissError(): void {
    this.lastError = null;
  }

  close(): void {
    this.#intentionalClose = true;
    window.removeEventListener("online", this.#onOnline);
    if (this.#retryTimer !== null) {
      clearTimeout(this.#retryTimer);
      this.#retryTimer = null;
    }
    this.#socket.close();
  }
}

let instance: BlogSocket | null = null;

/** Lazy app-wide singleton; components share one connection. */
export function getSocket(): BlogSocket {
  instance ??= new BlogSocket(wsUrl);
  return instance;
}
