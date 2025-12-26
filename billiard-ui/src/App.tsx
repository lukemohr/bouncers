import { HealthStatus } from "./components/HealthStatus";
import { UnitSquareSimulation } from "./components/UnitSquareSimulation";
import './App.css'

function App() {
  return (
    <div
      style={{
        minHeight: "100vh",
        display: "flex",
        flexDirection: "column",
        fontFamily: "system-ui, sans-serif",
      }}
    >
      <header
        style={{
          padding: "1rem 2rem",
          borderBottom: "1px solid #ddd",
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
        }}
      >
        <h1 style={{ margin: 0 }}>Bouncers</h1>
        <HealthStatus />
        <span style={{ fontSize: "0.9rem", color: "#555" }}>
          Rust billiards & React playground
        </span>
      </header>

      <main
        style={{
          flex: 1,
          display: "flex",
          padding: "1rem 2rem",
          gap: "1rem",
        }}
      >
        <section
          style={{
            flex: 2,
            border: "1px solid #ddd",
            borderRadius: "8px",
            padding: "1rem",
          }}
        >
          <h2>Table & Trajectory</h2>
          <p style={{ color: "#666" }}>
            Visualization will go here (SVG canvas).
          </p>
        </section>

        <section
          style={{
            flex: 1,
            border: "1px solid #ddd",
            borderRadius: "8px",
            padding: "1rem",
          }}
        >
          <h2>Controls</h2>
          <UnitSquareSimulation />
        </section>
      </main>
    </div>
  );
}

export default App;