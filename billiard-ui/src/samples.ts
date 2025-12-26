import type {
    BoundarySpec,
    TableSpec,
    SimulateRequest,
    BoundaryStateDto,
} from "./apiTypes";

export function unitSquareBoundary(name: string = "outer"): BoundarySpec {
    return {
        name,
        segments: [
            {
                kind: "line",
                start: { x: 0.0, y: 0.0 },
                end: { x: 1.0, y: 0.0 },
            },
            {
                kind: "line",
                start: { x: 1.0, y: 0.0 },
                end: { x: 1.0, y: 1.0 },
            },
            {
                kind: "line",
                start: { x: 1.0, y: 1.0 },
                end: { x: 0.0, y: 1.0 },
            },
            {
                kind: "line",
                start: { x: 0.0, y: 1.0 },
                end: { x: 0.0, y: 0.0 },
            },
        ],
    };
}

export function unitSquareTableSpec(): TableSpec {
    return {
        outer: unitSquareBoundary("outer"),
        obstacles: [],
    };
}

export function verticalOrbitInitialState(): BoundaryStateDto {
    return {
        component_index: 0,
        s: 0.5,
        theta: Math.PI / 2, // same as FRAC_PI_2 in Rust
    };
}

export function makeUnitSquareSimRequest(): SimulateRequest {
    return {
        table: unitSquareTableSpec(),
        initial_state: verticalOrbitInitialState(),
        max_steps: 4,
        epsilon: 1e-8,
    };
}