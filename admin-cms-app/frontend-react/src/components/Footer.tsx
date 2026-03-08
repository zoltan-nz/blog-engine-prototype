import { useHealthz } from "~/generated-api";
import { backendURL } from "~/lib/api/fetch-with-base-url";

export default function Footer() {
  const query = useHealthz();

  return (
    <footer
      className="border-t border-base-300 bg-base-100 px-8 py-4"
      data-testid="footer"
    >
      <div className="mx-auto flex max-w-4xl items-center justify-between text-sm text-base-content">
        <span className="text-base-content/50">Backend: {backendURL}</span>
        <span className="text-base-content/50">
          Server Name: {query.data?.data.meta.serverName}
        </span>
        <span className="text-base-content/50">
          Version: {query.data?.data.meta.version}
        </span>
        <button
          className="flex items-center gap-2 transition-opacity hover:opacity-80"
          onClick={() => query.refetch()}
          disabled={query.isPending}
          title={query.isSuccess ? "Connected" : "Disconnected"}
        >
          {query.isPending ? (
            <span className="loading loading-xs loading-spinner"></span>
          ) : (
            <span
              className={`h-3 w-3 rounded-full ${query.isSuccess ? "bg-success" : "bg-error"}`}
            ></span>
          )}
          <span className="text-base-content/60">
            {query.isSuccess ? "Connected" : "Disconnected"}
          </span>
        </button>
      </div>
    </footer>
  );
}
