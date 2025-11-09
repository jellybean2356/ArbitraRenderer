# Arbitra Rendering Engine

A compact renderer written in Rust using `wgpu` + `winit`. This repository is a student test project intended for learning and inspiration. It is not production-ready — feel free to read, study, and reuse ideas, but avoid using the code as-is in production.

---

## Table of contents

- [About](#about)
- [Features](#features)
- [Project layout](#project-layout)
- [How it works (high level)](#how-it-works-high-level)
- [Build & run (Windows PowerShell)](#build--run-windows-powershell)
- [Development notes & conventions](#development-notes--conventions)
- [Troubleshooting](#troubleshooting)
- [License](#license)

---

## About

This repository contains a single small rendering engine made in rust. The goal is to explore GPU concepts (surfaces, pipelines, vertex buffers, WGSL shaders) with clear, compact code. Expect informal error handling and simplified patterns chosen for clarity rather than robustness.

## Features

- Minimal rendering loop using `winit` and `wgpu`.
- Example WGSL shader(s) included with the binary via `include_str!(...)`.
- Object and scene handling with custom human-readable file formats (`.arobj`, `.arsc`).
- Transform system with position, rotation, and scale support.
- Per-instance rendering with model matrices uploaded to GPU uniform buffers.
- Camera system with view-projection matrix.
- Lightweight and easy to read — aimed at learning how the pieces fit together.

## Project layout

- `root/`
  - `Cargo.toml` — crate manifest and dependency list.
  - `src/` — source files: `main.rs`, `renderer.rs`, `vertex.rs`, `camera.rs`, `input.rs`, `transform.rs`, `object.rs`, `scene.rs`.
  - `src/shaders/` — WGSL shader files (e.g. `shader.wgsl`).
  - `assets/` — asset files for the engine.
    - `objects/` — object geometry files (`.arobj` format).
    - `scenes/` — scene definition files (`.arsc` format).

Open `/src/main.rs` to see the app lifecycle and pipeline setup. Shaders live in `/src/shaders/` and are compiled into the binary via `include_str!(...)` so editing requires recompilation.

## How it works (high level)

1. The program creates a `winit` window and queries an adapter through `wgpu`.
2. A `State` struct initializes the device, queue, surface, render pipeline, and GPU buffers.
3. Scene files (`.arsc`) are loaded, which reference object geometry files (`.arobj`).
4. Each object instance has a `Transform` (position, rotation, scale) that gets converted to a model matrix.
5. Per-instance uniform buffers are created to upload model matrices to the GPU.
6. The render loop listens for `winit` events (resize, input, redraw). On redraw the pipeline iterates over all scene instances, binding each model matrix and drawing the corresponding geometry.
7. Shaders are simple WGSL programs with `vs_main` and `fs_main` entry points — the vertex shader applies both the model and view-projection matrices.

### Architecture overview

- **Transform** (`transform.rs`): Encapsulates position, rotation, and scale; provides matrix conversion.
- **ObjectGeometry** (`object.rs`): Stores vertex and index data; loaded from `.arobj` files.
- **Scene** (`scene.rs`): Contains a list of `ObjectInstance` structs (geometry reference + transform + name); loaded from `.arsc` files.
- **Renderer** (`renderer.rs`): Manages GPU state, creates per-instance buffers and bind groups, executes draw calls.
- **Shaders** (`shaders/shader.wgsl`): WGSL vertex/fragment shaders with camera uniform (group 0) and per-instance model uniform (group 1).

### Custom file formats

#### `.arobj` — Object geometry format

Human-readable format for defining 3D object geometry:

```
name CubeName
vertices 8
position: -0.5 -0.5 -0.5  color: 1.0 0.0 0.0
position:  0.5 -0.5 -0.5  color: 0.0 1.0 0.0
...
indices 36
0 1 2
2 3 0
...
```

- `name <ObjectName>`: Optional object name.
- `vertices <count>`: Number of vertices.
- Each vertex line: `position: x y z  color: r g b`.
- `indices <count>`: Number of indices (must be multiple of 3 for triangles).
- Each index line: three indices forming a triangle (counter-clockwise winding).

#### `.arsc` — Scene format

Human-readable format for defining scenes with multiple object instances:

```
object
geometry: assets/objects/cube.arobj
name: RedCube
position: -2.0 0.0 0.0
rotation: 0.0 0.0 0.0
scale: 1.0 1.0 1.0

object
geometry: assets/objects/cube.arobj
name: GreenCube
position: 2.0 0.0 0.0
rotation: 0.0 45.0 0.0
scale: 1.5 1.5 1.5
```

- Each `object` block defines an instance.
- `geometry:` path to the `.arobj` file (relative to project root).
- `name:` instance name.
- `position:` x y z translation.
- `rotation:` x y z Euler angles in degrees.
- `scale:` x y z scale factors.

## Build & run (Windows PowerShell)

Run from the repository root.

```powershell
# From repo root
cargo run

# Enable runtime logging at info level
$env:RUST_LOG = "info"; cargo run

# Build only
cargo build
```

Notes

- Ensure your GPU drivers are up-to-date. `wgpu` will pick a suitable backend available on the host (DirectX / Vulkan / Metal depending on OS and configuration).
- Editing shaders requires recompilation because they are included at compile time.

## Development notes & conventions

- Shaders: add WGSL shader files under `engine/src/shaders/` and reference them using `include_str!("shaders/<file>.wgsl")`.
- Vertex layout: keep the Rust `Vertex` struct and its `Vertex::desc()` in sync with the WGSL `@location` attributes.
- Object files: create `.arobj` files in `assets/objects/` with the human-readable labeled format (see format specification above).
- Scene files: create `.arsc` files in `assets/scenes/` to define object instances with transforms.
- Per-instance rendering: each object instance gets its own uniform buffer for the model matrix to avoid GPU write conflicts.
- Async setup: `State::new` uses `pollster::block_on` to keep initialization simple. It's fine for a learning project; for production consider a fully async initialization.
- Error handling: the code may use `unwrap()` in a few places to keep examples concise. Replace with proper error handling for production use.

## Troubleshooting

- Black window / no draw: verify the adapter selection and surface format in `State::new` and ensure the GPU supports the selected features.
- Compile errors after shader changes: confirm WGSL entry point names (`vs_main`, `fs_main`) and matching attribute locations.
- On Windows, if `wgpu` fails to initialize, check that your graphics drivers are installed and that any required system dependencies are present.

## License

This project includes an `/LICENSE` file (MIT). The code is provided for educational use and inspiration; if you plan to reuse substantial portions, please include attribution.