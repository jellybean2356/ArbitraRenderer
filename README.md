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
- Simple vertex/index buffer setup and a single render pipeline.
- Lightweight and easy to read — aimed at learning how the pieces fit together.

## Project layout

- `root/`
  - `Cargo.toml` — crate manifest and dependency list.
  - `src/` — source files: `main.rs`, `renderer.rs`, `vertex.rs`, `camera.rs`, `input.rs` (where present).
  - `src/shaders/` — WGSL shader files (e.g. `shader.wgsl`).

Open `/src/main.rs` to see the app lifecycle and pipeline setup. Shaders live in `/src/shaders/` and are compiled into the binary via `include_str!(...)` so editing requires recompilation.

## How it works (high level)

1. The program creates a `winit` window and queries an adapter through `wgpu`.
2. A `State` struct initializes the device, queue, surface, render pipeline, and GPU buffers.
3. Vertex data is defined in Rust with `bytemuck` conversions so it can be uploaded to GPU buffers safely.
4. The render loop listens for `winit` events (resize, input, redraw). On redraw the pipeline draws the configured vertex/index buffers.
5. Shaders are simple WGSL programs with `vs_main` and `fs_main` entry points — update shader files and corresponding pipeline entry names together.

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
- Async setup: `State::new` uses `pollster::block_on` to keep initialization simple. It's fine for a learning project; for production consider a fully async initialization.
- Error handling: the code may use `unwrap()` in a few places to keep examples concise. Replace with proper error handling for production use.

## Troubleshooting

- Black window / no draw: verify the adapter selection and surface format in `State::new` and ensure the GPU supports the selected features.
- Compile errors after shader changes: confirm WGSL entry point names (`vs_main`, `fs_main`) and matching attribute locations.
- On Windows, if `wgpu` fails to initialize, check that your graphics drivers are installed and that any required system dependencies are present.

## License

This project includes an `/LICENSE` file (MIT). The code is provided for educational use and inspiration; if you plan to reuse substantial portions, please include attribution.