// Mirrors billiard_core::geometry::primitives::Vec2
export interface Vec2 {
    x: number;
    y: number;
}

// Mirrors SegmentSpec (with serde tag kind = "line" | "circular_arc")
export type SegmentSpec =
    | {
        kind: "line";
        start: Vec2;
        end: Vec2;
    }
    | {
        kind: "circular_arc";
        center: Vec2;
        radius: number;
        start_angle: number;
        end_angle: number;
        ccw: boolean;
    };

// Mirrors BoundarySpec
export interface BoundarySpec {
    name: string;
    segments: SegmentSpec[];
}

// Mirrors TableSpec
export interface TableSpec {
    outer: BoundarySpec;
    obstacles: BoundarySpec[];
}

// Mirrors BoundaryStateDto
export interface BoundaryStateDto {
    component_index: number;
    s: number;
    theta: number;
}

// Mirrors SimulateRequest
export interface SimulateRequest {
    table: TableSpec;
    initial_state: BoundaryStateDto;
    max_steps: number;
    epsilon: number;
}

// Mirrors CollisionDto
export interface CollisionDto {
    step: number;
    component_index: number;
    segment_index: number;
    s: number;
    theta: number;
    x: number;
    y: number;
}

// Mirrors SimulateResponse
export interface SimulateResponse {
    collisions: CollisionDto[];
}

// Health response (from /health)
export interface HealthResponse {
    status: string;
}