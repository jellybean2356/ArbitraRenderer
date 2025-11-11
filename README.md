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
- **Material system** with custom `.armat` format for defining materials with textures and PBR properties.
- **Texture support** with per-instance texture binding using the `image` crate for loading PNG files.
- **Lighting system** with directional light (sun) and up to 8 dynamic point lights.
- **Emissive objects** that automatically generate colored point lights with distance attenuation.
- Object and scene handling with custom human-readable file formats (`.arobj` metadata, `.arsc` scenes, `.armat` materials).
- Transform system with position, rotation, and scale support.
- Per-instance rendering with model matrices, emissive data, and textures uploaded to GPU.
- FPS camera system with mouse look and WASD movement.
- Depth buffer with backface culling for correct 3D rendering.
- Lightweight and easy to read — aimed at learning how the pieces fit together.

## Project layout

- `root/`
  - `Cargo.toml` — crate manifest and dependency list.
  - `src/` — source files: `main.rs`, `renderer.rs`, `vertex.rs`, `camera.rs`, `input.rs`, `transform.rs`, `object.rs`, `scene.rs`, `material.rs`, `texture.rs`.
  - `src/shaders/` — WGSL shader files (e.g. `shader.wgsl`).
  - `assets/` — asset files for the engine.
    - `models/` — 3D model files in standard OBJ format.
    - `objects/` — object metadata files (`.arobj` format) that reference OBJ models.
    - `scenes/` — scene definition files (`.arsc` format) with object instances, materials, and lighting settings.
    - `materials/` — material definition files (`.armat` format) with texture paths and PBR properties.
    - `textures/` — texture image files (PNG format) referenced by materials.

Open `/src/main.rs` to see the app lifecycle and pipeline setup. Shaders live in `/src/shaders/` and are compiled into the binary via `include_str!(...)` so editing requires recompilation.

## How it works (high level)

1. The program creates a `winit` window and queries an adapter through `wgpu`.
2. A `State` struct initializes the device, queue, surface, render pipeline, and GPU buffers.
3. Scene files (`.arsc`) are loaded, which reference object metadata files (`.arobj`) that point to standard OBJ model files and material files (`.armat`).
4. OBJ files are parsed using the `tobj` crate to extract vertex positions, normals, and UV coordinates.
5. Material files are loaded, which specify texture paths and PBR properties (roughness, metallic).
6. Textures are loaded from PNG files using the `image` crate and uploaded to GPU as texture arrays with samplers.
7. Each object instance has a `Transform` (position, rotation, scale) that gets converted to a model matrix, plus a material reference and optional emissive properties.
8. The scene's directional light settings are loaded and sent to the GPU as a uniform buffer.
9. Emissive objects are automatically converted to colored point lights with distance-based attenuation (up to 8 point lights).
10. Per-instance uniform buffers and texture bind groups are created to upload model matrices, emissive data, and textures to the GPU.
11. The render loop listens for `winit` events (resize, input, redraw). On redraw the pipeline iterates over all scene instances, binding each model matrix, texture, and drawing the corresponding geometry.
12. Shaders sample textures, calculate lighting from the directional light, all active point lights, and add emissive glow to objects that emit light.

### Architecture overview

- **Transform** (`transform.rs`): Encapsulates position, rotation, and scale; provides matrix conversion.
- **ObjectGeometry** (`object.rs`): Stores vertex and index data; loads OBJ files via `tobj` and parses `.arobj` metadata.
- **Vertex** (`vertex.rs`): GPU vertex structure with position, color (forced white), normal, and UV coordinates.
- **Material** (`material.rs`): Defines material properties loaded from `.armat` files (albedo texture path, roughness, metallic).
- **Texture** (`texture.rs`): Manages GPU texture resources; loads PNG images using the `image` crate and creates texture views and samplers.
- **Scene** (`scene.rs`): Contains a list of `ObjectInstance` structs (geometry reference + transform + name + emissive + emissive_color + material) and global `Light` settings; loaded from `.arsc` files.
- **Renderer** (`renderer.rs`): Manages GPU state, creates per-instance buffers and bind groups (camera, model, light, texture), executes draw calls with texture binding.
- **Camera** (`camera.rs`): FPS camera with mouse look and WASD movement; generates view-projection matrix.
- **Shaders** (`shaders/shader.wgsl`): WGSL vertex/fragment shaders with texture sampling and lighting calculations (ambient + diffuse from directional light + point lights + emissive glow).

### Lighting system

The engine supports two types of lighting:

- **Directional Light** (sun): Global light with direction, color, intensity, and ambient strength. Configured in `.arsc` scene files.
- **Point Lights** (up to 8): Automatically created from objects with `emissive > 0.0`. Each point light has position, color (from `emissive_color`), and intensity with distance attenuation (inverse square law).

Lighting calculations in the fragment shader:
1. Texture sampling (albedo from material texture)
2. Ambient light (base illumination)
3. Directional diffuse (based on surface normal and light direction)
4. Point light diffuse (for each point light, with distance falloff and colored lighting)
5. Emissive glow (added directly to fragment color for glowing objects)

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

#### `.armat` — Material format

Material definition files that specify texture and PBR properties:

```
name MaterialName
albedo_texture: textures/texture_name.png
roughness: 0.5
metallic: 0.0
```

- `name <MaterialName>`: Material identifier.
- `albedo_texture: <path>`: Relative path to the texture image file (PNG format).
- `roughness: <value>`: Surface roughness (0.0 = smooth, 1.0 = rough).
- `metallic: <value>`: Metallic property (0.0 = dielectric, 1.0 = metal).

**Note**: Currently, only `albedo_texture` is used in rendering. Roughness and metallic values are loaded but not yet applied in shaders.

**Built-in materials** (in `assets/materials/`):
- `white.armat` — Solid white texture
- `red.armat` — Solid red texture
- `green.armat` — Solid green texture
- `checkerboard.armat` — Black and white checkerboard pattern
- `sample_grid.armat` — Dark stone texture with grid pattern (for floors/ground)

#### `.arsc` — Scene format

Human-readable format for defining scenes with multiple object instances and lighting settings:

```
scene_name MyScene

light_direction: 0.3 -1.0 0.5
light_color: 1.0 1.0 0.9
light_intensity: 0.8
ambient_strength: 0.1

object
    geometry: objects/cube.arobj
    name: RedCube
    position: -2.0 0.0 0.0
    rotation: 0.0 0.0 0.0
    scale: 1.0 1.0 1.0
    material: materials/red.armat

object
    geometry: objects/pyramid.arobj
    name: GlowingPyramid
    position: 2.0 1.0 0.0
    rotation: 0.0 45.0 0.0
    scale: 1.0 1.0 1.0
    emissive: 1.5
    emissive_color: 1.0 0.3 0.3
    material: materials/red.armat
```

**Scene settings**:
- `scene_name <name>`: Optional scene name for identification.

**Light settings** (global directional light):
- `light_direction:` x y z vector (doesn't need to be normalized).
- `light_color:` r g b color (0.0 to 1.0 range).
- `light_intensity:` brightness multiplier.
- `ambient_strength:` minimum ambient illumination (0.0 = pitch black in shadows, 1.0 = fully lit everywhere).

**Object blocks**:
- `geometry:` path to the `.arobj` metadata file (relative to assets folder, e.g., `objects/cube.arobj`).
- `name:` instance name for identification.
- `position:` x y z translation.
- `rotation:` x y z Euler angles in degrees.
- `scale:` x y z scale factors.
- `emissive:` (optional) how much light the object emits (0.0 = none, higher values = brighter point light). Objects with `emissive > 0.0` automatically create point lights at their position.
- `emissive_color:` (optional) r g b color of the emitted light (default: 1.0 1.0 1.0 for white). Determines the color of the point light generated by emissive objects.
- `material:` path to the `.armat` material file (relative to assets folder, e.g., `materials/red.armat`).

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

- Shaders: add WGSL shader files under `src/shaders/` and reference them using `include_str!("shaders/<file>.wgsl")`.
- Vertex layout: keep the Rust `Vertex` struct and its `Vertex::desc()` in sync with the WGSL `@location` attributes.
- **Models**: Create standard OBJ files in `assets/models/` with positions, normals, and UV coordinates.
- **Object metadata**: Create `.arobj` files in `assets/objects/` that reference OBJ models (see format specification above).
- **Materials**: Create `.armat` files in `assets/materials/` with texture paths and PBR properties. Materials are defined per-instance (not per-geometry).
- **Textures**: Place PNG texture files in `assets/textures/` and reference them from material files.
- **Scenes**: Create `.arsc` files in `assets/scenes/` to define object instances with transforms, materials, emissive values, and global lighting settings.
- **Lighting**: Set directional light parameters in the scene file. Objects with `emissive > 0.0` automatically become colored point lights based on their `emissive_color`.
- **Point lights**: Maximum of 8 point lights per scene (limitation set in `renderer.rs`). The engine uses emissive objects to generate point lights with color and intensity.
- **Bind groups**: The engine uses 4 bind groups (0=Camera, 1=Model, 2=Light, 3=Texture). This is a hardware limitation that requires careful management.
- Per-instance rendering: each object instance gets its own uniform buffer for the model matrix and emissive value, plus a texture bind group.
- **Struct padding**: GPU uniform structs must match WGSL layout exactly. Use explicit padding fields when needed (see `LightUniform`, `ModelUniform`, `PointLight` in `renderer.rs`).
- Async setup: `State::new` uses `pollster::block_on` to keep initialization simple. It's fine for a learning project; for production consider a fully async initialization.
- Error handling: the code may use `unwrap()` in a few places to keep examples concise. Replace with proper error handling for production use.

## Troubleshooting

- Black window / no draw: verify the adapter selection and surface format in `State::new` and ensure the GPU supports the selected features.
- Compile errors after shader changes: confirm WGSL entry point names (`vs_main`, `fs_main`) and matching attribute locations.
- On Windows, if `wgpu` fails to initialize, check that your graphics drivers are installed and that any required system dependencies are present.

## License

This project includes an `/LICENSE` file (MIT). The code is provided for educational use and inspiration; if you plan to reuse substantial portions, please include attribution.