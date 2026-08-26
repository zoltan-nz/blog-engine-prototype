export type PostDraft = { title: string; description: string; body: string };
export type SaveFn = (payload: PostDraft) => Promise<void>;
export type SaveStatus = "idle" | "saving" | "saved" | "error";

/**
 * Debounces edits into `save` calls and guarantees at most one in-flight
 * call at a time. Edits that land while a save is in flight coalesce into
 * exactly one follow-up save fired right after the current one settles —
 * this is what keeps concurrent `UpdatePost` sends from racing (see the
 * spec's ordering invariant).
 */
export function createAutosave(save: SaveFn, debounceMs = 500) {
  let latestDraft: PostDraft | null = null;
  let timer: ReturnType<typeof setTimeout> | null = null;
  let saving = false;
  let pending = false;
  let status = $state<SaveStatus>("idle");

  function scheduleDebounced(): void {
    if (timer !== null) clearTimeout(timer);
    timer = setTimeout(() => {
      timer = null;
      void performSave();
    }, debounceMs);
  }

  async function performSave(): Promise<void> {
    if (saving) {
      pending = true;
      return;
    }
    if (latestDraft === null) return;

    saving = true;
    status = "saving";
    const draft = latestDraft;
    try {
      await save(draft);
      status = "saved";
    } catch {
      status = "error";
    } finally {
      saving = false;
    }

    if (pending) {
      pending = false;
      await performSave();
    }
  }

  return {
    update(draft: PostDraft): void {
      latestDraft = draft;
      scheduleDebounced();
    },
    get status(): SaveStatus {
      return status;
    },
    async flush(): Promise<void> {
      if (timer !== null) {
        clearTimeout(timer);
        timer = null;
      }
      await performSave();
    },
  };
}
