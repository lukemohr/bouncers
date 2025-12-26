import { useEffect, useState } from "react";
import type { HealthResponse } from "../apiTypes";

type Status = "idle" | "loading" | "success" | "error";

export function HealthStatus() {
    const [status, setStatus] = useState<Status>("idle");
    const [message, setMessage] = useState<string | null>(null);

    useEffect(() => {
        let cancelled = false;

        async function checkHealth() {
            setStatus("loading");
            setMessage(null);

            try {
                const res = await fetch("http://127.0.0.1:3000/health");
                if (!res.ok) {
                    throw new Error(`HTTP ${res.status}`);
                }
                const data = (await res.json()) as HealthResponse;
                if (!cancelled) {
                    setStatus("success");
                    setMessage(`API status: ${data.status}`);
                }
            } catch (err: unknown) {
                if (!cancelled) {
                    setStatus("error");
                    const msg =
                        err instanceof Error ? err.message : "Unknown error while calling /health";
                    setMessage(msg);
                }
            }
        }

        checkHealth();

        // Cleanup in case component unmounts while request is in flight
        return () => {
            cancelled = true;
        };
    }, []);

    let color = "#666";
    if (status === "loading") color = "#8888ff";
    if (status === "success") color = "#22aa55";
    if (status === "error") color = "#cc3344";

    return (
        <div style={{ fontSize: "0.9rem", color }}>
            <strong>Backend health:</strong>{" "}
            {status === "idle" && "idle"}
            {status === "loading" && "checking..."}
            {status === "success" && message}
            {status === "error" && `error: ${message}`}
        </div>
    );
}