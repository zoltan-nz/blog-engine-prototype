import type { Route } from "./+types/home";

export function meta({}: Route.MetaArgs) {
  return [
    { title: "Blog Engine Admin - React Client" },
    { name: "description", content: "Manage your Astro blog sites." },
  ];
}

export default function Home() {
  return (
    <main className="flex-1 p-8">
      <div className="mx-auto max-w-4xl">
        <h1 className="mb-4 text-3xl font-bold">Blog Engine Admin</h1>
        <p className="text-base-content/70">Manage your Astro blog sites.</p>
      </div>
    </main>
  );
}
