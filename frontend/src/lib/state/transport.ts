import { Channel, invoke, isTauri } from "@tauri-apps/api/core";
import type { WsEnvelope } from "#lib/types/bindings.js";

/** What a transport reports back to `BlogSocket`. */
export interface TransportHandlers {
  onOpen(): void;
  onClose(): void;
  onMessage(envelope: WsEnvelope): void;
}

/** One connection's outbound half. `send` drops envelopes while not open. */
export interface Transport {
  send(envelope: WsEnvelope): void;
  close(): void;
}

export type OpenTransport = (handlers: TransportHandlers) => Transport;

/** Browser path: the backend's `/ws` endpoint. */
export function webSocketTransport(url: string): OpenTransport {
  return (handlers) => {
    const socket = new WebSocket(url);
    socket.onopen = () => handlers.onOpen();
    socket.onclose = () => handlers.onClose();
    socket.onerror = (error) => console.error(error);
    socket.onmessage = (event) => {
      try {
        handlers.onMessage(JSON.parse(event.data));
      } catch (err) {
        console.error("Failed to parse WS message", err);
      }
    };

    return {
      send(envelope) {
        if (socket.readyState === WebSocket.OPEN) {
          socket.send(JSON.stringify(envelope));
        }
      },
      close() {
        socket.close();
      },
    };
  };
}

/** Desktop path: the `connect` / `send` commands in `src-tauri`. The backend
 * runs in-process, so there is no socket to drop; a failed `connect` is
 * reported as a close so `BlogSocket` retries. */
export const ipcTransport: OpenTransport = (handlers) => {
  let closed = false;
  const onMessage = new Channel<WsEnvelope>((envelope) => {
    if (!closed) handlers.onMessage(envelope);
  });

  invoke("connect", { onMessage }).then(
    () => {
      if (!closed) handlers.onOpen();
    },
    (error) => {
      console.error("IPC connect failed", error);
      if (!closed) handlers.onClose();
    },
  );

  return {
    send(envelope) {
      invoke("send", { envelope: JSON.stringify(envelope) }).catch((error) =>
        console.error("IPC send failed", error),
      );
    },
    close() {
      closed = true;
    },
  };
};

/** IPC inside the Tauri webview, WebSocket everywhere else (dev, Playwright). */
export function defaultTransport(wsUrl: string): OpenTransport {
  return isTauri() ? ipcTransport : webSocketTransport(wsUrl);
}
