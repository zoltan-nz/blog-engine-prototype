import { env } from "$env/dynamic/public";
import type {
  Command,
  ErrorCode,
  Event,
  LogStream,
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

  #url: string;
  #socket!: WebSocket;
  #retryDelay = INITIAL_DELAY_MS;
  #retryTimer: ReturnType<typeof setTimeout> | null = null;
  #intentionalClose = false;

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
      case "Error":
        this.lastError = {
          code: event.payload.code,
          message: event.payload.message,
          correlationId: event.payload.correlation_id,
        };
        break;
      case "Pong":
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
