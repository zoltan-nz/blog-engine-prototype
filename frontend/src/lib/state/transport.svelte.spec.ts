import { beforeEach, describe, expect, it, vi } from "vitest";
import type { WsEnvelope } from "#lib/types/bindings.js";

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));

// Stand-in for Tauri's Channel: tests push messages through `onmessage`.
class FakeChannel<T> {
  onmessage: (message: T) => void;
  constructor(onmessage: (message: T) => void) {
    this.onmessage = onmessage;
  }
}

vi.mock("@tauri-apps/api/core", () => ({
  invoke,
  Channel: FakeChannel,
  isTauri: () => false,
}));

const { ipcTransport } = await import("./transport.js");

const pong: WsEnvelope = {
  unix_timestamp_us: 1,
  correlation_id: "c1",
  type: "Event",
  payload: { type: "Pong" },
};

function handlers() {
  return { onOpen: vi.fn(), onClose: vi.fn(), onMessage: vi.fn() };
}

function connectChannel(): FakeChannel<WsEnvelope> {
  const [command, args] = invoke.mock.calls[0];
  expect(command).toBe("connect");
  return args.onMessage;
}

describe("ipcTransport", () => {
  beforeEach(() => {
    invoke.mockReset();
  });

  it("opens once connect resolves and forwards channel messages", async () => {
    invoke.mockResolvedValue(undefined);
    const h = handlers();
    ipcTransport(h);

    await vi.waitFor(() => expect(h.onOpen).toHaveBeenCalledOnce());
    connectChannel().onmessage(pong);
    expect(h.onMessage).toHaveBeenCalledWith(pong);
  });

  it("sends envelopes as JSON through the send command", () => {
    invoke.mockResolvedValue(undefined);
    const transport = ipcTransport(handlers());

    transport.send(pong);
    expect(invoke).toHaveBeenLastCalledWith("send", {
      envelope: JSON.stringify(pong),
    });
  });

  it("reports a failed connect as closed so the socket retries", async () => {
    invoke.mockRejectedValue(new Error("boom"));
    const h = handlers();
    ipcTransport(h);

    await vi.waitFor(() => expect(h.onClose).toHaveBeenCalledOnce());
    expect(h.onOpen).not.toHaveBeenCalled();
  });

  it("ignores messages after close", () => {
    invoke.mockResolvedValue(undefined);
    const h = handlers();
    const transport = ipcTransport(h);

    transport.close();
    connectChannel().onmessage(pong);
    expect(h.onMessage).not.toHaveBeenCalled();
  });
});
