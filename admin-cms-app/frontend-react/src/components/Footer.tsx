import { useState, useEffect } from "react";

export default function Footer() {
  const backendUrl =
    import.meta.env.VITE_API_BACKEND_URL || "http://localhost:3001";

  const [connected, setConnected] = useState(false);
  const [loading, setLoading] = useState(true);

  const [version, setVersion] = useState(null);
  const [serverName, setServerName] = useState("");

  const checkConnection = async () => {
    setLoading(true);
    try {
      const response = await fetch(`${backendUrl}/healthz`);
      setConnected(response.ok);

      const data = await response.json();
      setVersion(data.meta.version);
      setServerName(data.meta.serverName);

    } catch (error) {
      console.error("Failed to connect to backend:", error);
      setConnected(false);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    (async () => await checkConnection())();
  }, []);

  return (
    <footer className="border-t border-base-300 bg-base-100 px-8 py-4" data-testid="footer">
      <div className="mx-auto flex max-w-4xl items-center justify-between text-sm text-base-content">
        <span className="text-base-content/50">Backend: {backendUrl}</span>
        <span className="text-base-content/50">Server Name: {serverName}</span>
        <span className="text-base-content/50">Version: {version}</span>
        <button
          className="flex items-center gap-2 transition-opacity hover:opacity-80"
          onClick={checkConnection}
          disabled={loading}
          title={connected ? "Connected" : "Disconnected"}
        >
          {loading ? (
            <span className="loading loading-xs loading-spinner"></span>
          ) : (
            <span
              className={`h-3 w-3 rounded-full ${connected ? "bg-success" : "bg-error"}`}
            ></span>
          )}
          <span className="text-base-content/60">
            {connected ? "Connected" : "Disconnected"}
          </span>
        </button>
      </div>
    </footer>
  );
}
