import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createAutosave, type PostDraft } from "./autosave.svelte";

const draft = (body: string): PostDraft => ({
  title: "Title",
  description: "Desc",
  body,
});

/** A save fn whose resolution the test controls explicitly. */
function deferredSave() {
  const calls: PostDraft[] = [];
  let resolveCurrent: (() => void) | null = null;
  let rejectCurrent: ((error: Error) => void) | null = null;

  const save = vi.fn((payload: PostDraft) => {
    calls.push(payload);
    return new Promise<void>((resolve, reject) => {
      resolveCurrent = resolve;
      rejectCurrent = reject;
    });
  });

  return {
    save,
    calls,
    resolve: () => resolveCurrent?.(),
    reject: (error: Error) => rejectCurrent?.(error),
  };
}

describe("createAutosave", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("coalesces edits within the debounce window into one save", async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    const autosave = createAutosave(save, 500);

    autosave.update(draft("v1"));
    autosave.update(draft("v2"));
    autosave.update(draft("v3"));
    await vi.advanceTimersByTimeAsync(500);

    expect(save).toHaveBeenCalledTimes(1);
    expect(save).toHaveBeenCalledWith(draft("v3"));
  });

  it("triggers exactly one follow-up save after edits arrive mid-save", async () => {
    const fake = deferredSave();
    const autosave = createAutosave(fake.save, 500);

    autosave.update(draft("v1"));
    await vi.advanceTimersByTimeAsync(500);
    expect(fake.save).toHaveBeenCalledTimes(1);

    // Edits land while the first save is still in flight.
    autosave.update(draft("v2"));
    autosave.update(draft("v3"));
    await vi.advanceTimersByTimeAsync(500);
    expect(fake.save).toHaveBeenCalledTimes(1); // still just the first — save not settled yet

    fake.resolve();
    await vi.waitFor(() => expect(fake.save).toHaveBeenCalledTimes(2));
    expect(fake.calls[1]).toEqual(draft("v3"));
  });

  it("sets status to error on a rejected save, and the next edit retries", async () => {
    const fake = deferredSave();
    const autosave = createAutosave(fake.save, 500);

    autosave.update(draft("v1"));
    await vi.advanceTimersByTimeAsync(500);
    fake.reject(new Error("network down"));
    await vi.waitFor(() => expect(autosave.status).toBe("error"));

    autosave.update(draft("v2"));
    await vi.advanceTimersByTimeAsync(500);

    expect(fake.save).toHaveBeenCalledTimes(2);
    expect(fake.calls[1]).toEqual(draft("v2"));
  });

  it("reports saving then saved for a normal debounced save", async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    const autosave = createAutosave(save, 500);

    expect(autosave.status).toBe("idle");
    autosave.update(draft("v1"));
    await vi.advanceTimersByTimeAsync(500);

    await vi.waitFor(() => expect(autosave.status).toBe("saved"));
  });

  it("flush saves immediately without waiting for the debounce", async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    const autosave = createAutosave(save, 500);

    autosave.update(draft("v1"));
    await autosave.flush();

    expect(save).toHaveBeenCalledTimes(1);
    expect(save).toHaveBeenCalledWith(draft("v1"));
  });
});
