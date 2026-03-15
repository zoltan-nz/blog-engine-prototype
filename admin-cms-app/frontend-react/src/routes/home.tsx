import { useRef, useState } from "react";
import type { Route } from "./+types/home";
import { useListSites, useCreateSite } from "~/generated-api";
import type { CreateSiteRequest } from "~/generated-api";

export function meta({}: Route.MetaArgs) {
  return [
    { title: "Blog Engine Admin - React Client" },
    { name: "description", content: "Manage your Astro blog sites." },
  ];
}

function nameToSlug(name: string): string {
  return name
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 100);
}

export default function Home() {
  const sitesQuery = useListSites();
  const createSiteMutation = useCreateSite();
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [blogName, setBlogName] = useState("");
  const slug = nameToSlug(blogName);

  function openModal() {
    setBlogName("");
    dialogRef.current?.showModal();
  }

  function closeModal() {
    dialogRef.current?.close();
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!blogName.trim() || !slug) return;

    const body: CreateSiteRequest = { name: blogName.trim(), slug };
    await createSiteMutation.mutateAsync({ data: body });

    closeModal();
    sitesQuery.refetch();
  }

  const sites = sitesQuery.data?.data.data ?? [];

  return (
    <main className="flex-1 p-8">
      <div className="mx-auto max-w-4xl">
        <div className="mb-8 flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold">Blog Engine Admin</h1>
            <p className="text-base-content/70 mt-1">Manage your Astro blog sites.</p>
          </div>
          <button className="btn btn-primary" onClick={openModal}>
            Create a new blog
          </button>
        </div>

        {/* Site list */}
        {sitesQuery.isLoading ? (
          <div className="flex justify-center py-12">
            <span className="loading loading-spinner loading-lg"></span>
          </div>
        ) : sitesQuery.isError ? (
          <div className="alert alert-error">
            <span>Failed to load sites. Is the backend running?</span>
          </div>
        ) : sites.length === 0 ? (
          <div className="card bg-base-200 py-16 text-center">
            <p className="text-base-content/60">No blogs yet. Create your first one!</p>
          </div>
        ) : (
          <ul className="grid gap-4 sm:grid-cols-2">
            {sites.map((site) => (
              <li key={site.slug} className="card bg-base-100 border-base-300 border shadow-sm">
                <div className="card-body">
                  <h2 className="card-title">{site.name}</h2>
                  <p className="text-base-content/60 font-mono text-sm">{site.slug}</p>
                  <p className="text-base-content/50 truncate text-xs">{site.gitUrl}</p>
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>

      {/* Create blog modal */}
      <dialog ref={dialogRef} className="modal">
        <div className="modal-box">
          <h3 className="mb-4 text-lg font-bold">Create a new blog</h3>

          <form onSubmit={handleSubmit}>
            <div className="form-control mb-4">
              <label className="label" htmlFor="blog-name">
                <span className="label-text">Blog name</span>
              </label>
              <input
                id="blog-name"
                type="text"
                placeholder="My Awesome Blog"
                className="input input-bordered w-full"
                value={blogName}
                onChange={(e) => setBlogName(e.target.value)}
                required
              />
            </div>

            <div className="form-control mb-6">
              <label className="label" htmlFor="blog-slug">
                <span className="label-text">Slug (auto-generated)</span>
              </label>
              <input
                id="blog-slug"
                type="text"
                className="input input-bordered w-full font-mono"
                value={slug}
                readOnly
              />
              <p className="label label-text-alt text-base-content/50">
                Used as directory and git repo name
              </p>
            </div>

            {createSiteMutation.isError && (
              <div className="alert alert-error mb-4">
                <span>Failed to create blog. Please try again.</span>
              </div>
            )}

            <div className="modal-action">
              <button type="button" className="btn btn-ghost" onClick={closeModal}>
                Cancel
              </button>
              <button
                type="submit"
                className="btn btn-primary"
                disabled={!blogName.trim() || createSiteMutation.isPending}
              >
                {createSiteMutation.isPending ? (
                  <>
                    <span className="loading loading-spinner loading-sm"></span>
                    Creating…
                  </>
                ) : (
                  "Create"
                )}
              </button>
            </div>
          </form>
        </div>
        <form method="dialog" className="modal-backdrop">
          <button>close</button>
        </form>
      </dialog>
    </main>
  );
}
