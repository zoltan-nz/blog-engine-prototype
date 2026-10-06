// Non-reactive stand-in for BlogSocket used by component tests. Tests assign
// to the fields directly before render.
import type { BuildLogLine, ProtocolError } from "$lib/state/socket.svelte";
import type { PostMeta, PreviewView, SiteView } from "$lib/types/bindings.js";
import { vi } from "vitest";

export const fakeSocket = {
  status: "open" as const,
  sites: [] as SiteView[],
  preview: {
    state: { type: "Stopped" },
    slug: null,
    url: null,
  } as PreviewView,
  buildLogs: [] as BuildLogLine[],
  lastError: null as ProtocolError | null,
  posts: {} as Record<string, PostMeta[]>,
  createSite: vi.fn(),
  buildSite: vi.fn(),
  startPreview: vi.fn(),
  stopPreview: vi.fn(),
  deleteSite: vi.fn(),
  ping: vi.fn(),
  requestPost: vi.fn(),
  createPost: vi.fn(),
  deletePost: vi.fn(),
  updatePost: vi.fn(),
  draftPost: vi.fn(),
  dismissError: vi.fn(),
  close: vi.fn(),
};
