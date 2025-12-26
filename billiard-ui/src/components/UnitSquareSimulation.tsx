import { useState } from "react";
import type { CollisionDto, SimulateResponse } from "../apiTypes";
import { makeUnitSquareSimRequest } from "../samples";

type SimStatus = "idle" | "running" | "success" | "error";

export function UnitSquareSimulation() {
    const [status, setStatus] = useState<SimStatus>("idle");
    const [error, setError] = useState<string | null>(null);
    const [collisions, setCollisions] = useState<CollisionDto[]>([]);

    async function runSimulation() {
        setStatus("running");
        setError(null);
        setCollisions([]);

        const payload = makeUnitSquareSimRequest();

        try {
            const res = await fetch("http://127.0.0.1:3000/simulate", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify(payload),
            });

            if (!res.ok) {
                const text = await res.text();
                throw new Error(`HTTP ${res.status}: ${text}`);
            }

            const data = (await res.json()) as SimulateResponse;
            setCollisions(data.collisions);
            setStatus("success");
        } catch (err: unknown) {
            const msg =
                err instanceof Error ? err.message : "Unknown error during simulation";
            setError(msg);
            setStatus("error");
        }
    }

    return (
        <div>
            <h3>Unit square vertical orbit demo</h3>
            <p style={{ fontSize: "0.9rem", color: "#555" }}>
                Uses  a unit square table,
                starting at (0.5, 0) with θ = π/2 (straight up). Expect bounce between
                top and bottom edges.
            </p>

            <button
                onClick={runSimulation}
                disabled={status === "running"}
                style={{
                    padding: "0.5rem 1rem",
                    borderRadius: "4px",
                    border: "1px solid #888",
                    backgroundColor: status === "running" ? "#ddd" : "#f5f5f5",
                    cursor: status === "running" ? "default" : "pointer",
                }}
            >
                {status === "running" ? "Running..." : "Run simulation"}
            </button>

            {status === "error" && (
                <p style={{ color: "#c33", marginTop: "0.5rem" }}>Error: {error}</p>
            )}

            {status === "success" && (
                <div style={{ marginTop: "1rem" }}>
                    <p>
                        Collisions: <strong>{collisions.length}</strong>
                    </p>
                    <table
                        style={{
                            width: "100%",
                            borderCollapse: "collapse",
                            fontSize: "0.85rem",
                        }}
                    >
                        <thead>
                            <tr>
                                <th style={{ borderBottom: "1px solid #ccc", textAlign: "left" }}>step</th>
                                <th style={{ borderBottom: "1px solid #ccc", textAlign: "left" }}>comp</th>
                                <th style={{ borderBottom: "1px solid #ccc", textAlign: "left" }}>seg</th>
                                <th style={{ borderBottom: "1px solid #ccc", textAlign: "left" }}>x</th>
                                <th style={{ borderBottom: "1px solid #ccc", textAlign: "left" }}>y</th>
                                <th style={{ borderBottom: "1px solid #ccc", textAlign: "left" }}>s</th>
                                <th style={{ borderBottom: "1px solid #ccc", textAlign: "left" }}>θ</th>
                            </tr>
                        </thead>
                        <tbody>
                            {collisions.map((c) => (
                                <tr key={c.step}>
                                    <td>{c.step}</td>
                                    <td>{c.component_index}</td>
                                    <td>{c.segment_index}</td>
                                    <td>{c.x.toFixed(3)}</td>
                                    <td>{c.y.toFixed(3)}</td>
                                    <td>{c.s.toFixed(3)}</td>
                                    <td>{c.theta.toFixed(3)}</td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            )}
        </div>
    );
}