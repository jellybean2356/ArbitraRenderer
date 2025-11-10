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
- **OBJ file loading** using the `tobj` crate for standard 3D model support.
- **Lighting system** with directional light (sun) and up to 8 dynamic point lights.
- **Emissive objects** that automatically generate point lights with distance attenuation.
- Object and scene handling with custom human-readable file formats (`.arobj` metadata, `.arsc` scenes).
- Transform system with position, rotation, and scale support.
- Per-instance rendering with model matrices and emissive data uploaded to GPU uniform buffers.
- FPS camera system with mouse look and WASD movement.
- UV coordinate support (ready for texture mapping).
- Depth buffer with backface culling for correct 3D rendering.
- Lightweight and easy to read — aimed at learning how the pieces fit together.

## Project layout

- `root/`
  - `Cargo.toml` — crate manifest and dependency list.
  - `src/` — source files: `main.rs`, `renderer.rs`, `vertex.rs`, `camera.rs`, `input.rs`, `transform.rs`, `object.rs`, `scene.rs`.
  - `src/shaders/` — WGSL shader files (e.g. `shader.wgsl`).
  - `assets/` — asset files for the engine.
    - `models/` — 3D model files in standard OBJ format.
    - `objects/` — object metadata files (`.arobj` format) that reference OBJ models.
    - `scenes/` — scene definition files (`.arsc` format) with object instances and lighting settings.

Open `/src/main.rs` to see the app lifecycle and pipeline setup. Shaders live in `/src/shaders/` and are compiled into the binary via `include_str!(...)` so editing requires recompilation.

## How it works (high level)

1. The program creates a `winit` window and queries an adapter through `wgpu`.
2. A `State` struct initializes the device, queue, surface, render pipeline, and GPU buffers.
3. Scene files (`.arsc`) are loaded, which reference object metadata files (`.arobj`) that point to standard OBJ model files.
4. OBJ files are parsed using the `tobj` crate to extract vertex positions, normals, and UV coordinates.
5. Each object instance has a `Transform` (position, rotation, scale) that gets converted to a model matrix.
6. The scene's directional light settings are loaded and sent to the GPU as a uniform buffer.
7. Emissive objects are automatically converted to point lights with distance-based attenuation (up to 8 point lights).
8. Per-instance uniform buffers are created to upload model matrices and emissive data to the GPU.
9. The render loop listens for `winit` events (resize, input, redraw). On redraw the pipeline iterates over all scene instances, binding each model matrix and drawing the corresponding geometry.
10. Shaders calculate lighting from the directional light, all active point lights, and add emissive glow to objects that emit light.

### Architecture overview

- **Transform** (`transform.rs`): Encapsulates position, rotation, and scale; provides matrix conversion.
- **ObjectGeometry** (`object.rs`): Stores vertex and index data; loads OBJ files via `tobj` and parses `.arobj` metadata.
- **Vertex** (`vertex.rs`): GPU vertex structure with position, color (forced white for now), normal, and UV coordinates.
- **Scene** (`scene.rs`): Contains a list of `ObjectInstance` structs (geometry reference + transform + name + emissive) and global `Light` settings; loaded from `.arsc` files.
- **Renderer** (`renderer.rs`): Manages GPU state, creates per-instance buffers and bind groups (camera, model, directional light, point lights), executes draw calls.
- **Camera** (`camera.rs`): FPS camera with mouse look and WASD movement; generates view-projection matrix.
- **Shaders** (`shaders/shader.wgsl`): WGSL vertex/fragment shaders with lighting calculations (ambient + diffuse from directional light + point lights + emissive glow).

### Lighting system

The engine supports two types of lighting:

- **Directional Light** (sun): Global light with direction, color, intensity, and ambient strength. Configured in `.arsc` scene files.
- **Point Lights** (up to 8): Automatically created from objects with `emissive > 0.0`. Each point light has position, color, and intensity with distance attenuation (inverse square law).

Lighting calculations in the fragment shader:
1. Ambient light (base illumination)
2. Directional diffuse (based on surface normal and light direction)
3. Point light diffuse (for each point light, with distance falloff)
4. Emissive glow (added directly to fragment color for glowing objects)

### Custom file formats

#### `.arobj` — Object metadata format

Lightweight metadata files that reference standard OBJ model files:

```
name CubeName
obj_file: models/cube.obj
```

- `name <ObjectName>`: Optional object name for identification.
- `obj_file: <path>`: Relative path to the OBJ file (from project root).

The OBJ file itself contains standard Wavefront OBJ data (vertices, normals, UVs, faces). The engine uses the `tobj` crate to parse OBJ files and extract mesh data.

#### `.arsc` — Scene format

Human-readable format for defining scenes with multiple object instances and lighting settings:

```
light_direction: 0.3 -1.0 0.5
light_color: 1.0 1.0 0.9
light_intensity: 0.8
light_ambient_strength: 0.1

object
geometry: assets/objects/cube.arobj
name: RedCube
position: -2.0 0.0 0.0
rotation: 0.0 0.0 0.0
scale: 1.0 1.0 1.0
emissive: 0.0

object
geometry: assets/objects/pyramid.arobj
name: GlowingPyramid
position: 2.0 1.0 0.0
rotation: 0.0 45.0 0.0
scale: 1.0 1.0 1.0
emissive: 0.5
```

**Light settings** (global directional light):
- `light_direction:` x y z vector (doesn't need to be normalized).
- `light_color:` r g b color (0.0 to 1.0 range).
- `light_intensity:` brightness multiplier.
- `light_ambient_strength:` minimum ambient illumination (0.0 = pitch black in shadows, 1.0 = fully lit everywhere).

**Object blocks**:
- `geometry:` path to the `.arobj` metadata file (relative to project root).
- `name:` instance name for identification.
- `position:` x y z translation.
- `rotation:` x y z Euler angles in degrees.
- `scale:` x y z scale factors.
- `emissive:` how much light the object emits (0.0 = none, higher values = brighter point light). Objects with `emissive > 0.0` automatically create point lights at their position.

## Build & run (Windows PowerShell)

**Important**: Always run commands from the directory containing `Cargo.toml` (the repository root), as the application expects to find the `assets/` folder relative to the current working directory.

```powershell
# Run the application (from repository root)
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
- **Models**: Create standard OBJ files in `assets/models/` with positions, normals, and UV coordinates.
- **Object metadata**: Create `.arobj` files in `assets/objects/` that reference OBJ models (see format specification above).
- **Scenes**: Create `.arsc` files in `assets/scenes/` to define object instances with transforms, emissive values, and global lighting settings.
- **Lighting**: Set directional light parameters in the scene file. Objects with `emissive > 0.0` automatically become point lights.
- **Point lights**: Maximum of 8 point lights per scene (limitation set in `renderer.rs`). The engine uses emissive objects to generate point lights.
- Per-instance rendering: each object instance gets its own uniform buffer for the model matrix and emissive value to avoid GPU write conflicts.
- **Struct padding**: GPU uniform structs must match WGSL layout exactly. Use explicit padding fields when needed (see `LightUniform`, `ModelUniform`, `PointLight` in `renderer.rs`).
- Async setup: `State::new` uses `pollster::block_on` to keep initialization simple. It's fine for a learning project; for production consider a fully async initialization.
- Error handling: the code may use `unwrap()` in a few places to keep examples concise. Replace with proper error handling for production use.

## Troubleshooting

- Black window / no draw: verify the adapter selection and surface format in `State::new` and ensure the GPU supports the selected features.
- Compile errors after shader changes: confirm WGSL entry point names (`vs_main`, `fs_main`) and matching attribute locations.
- On Windows, if `wgpu` fails to initialize, check that your graphics drivers are installed and that any required system dependencies are present.

## License

This project includes an `/LICENSE` file (MIT). The code is provided for educational use and inspiration; if you plan to reuse substantial portions, please include attribution.