# ArbitraRenderer (student test project)

> Learning renderer built with Rust + wgpu — student test project. Use for inspiration and learning only; not intended as production-quality code or a template for redistribution.

## What this repository contains

- `engine/` — the single binary crate that implements a tiny renderer using `wgpu` and `winit`.
  - `engine/src/main.rs` — application entry, `State` lifecycle, render loop and pipeline setup.
  - `engine/src/shaders/` — WGSL shader(s) included at compile time (e.g. `shader.wgsl`).
  - `engine/Cargo.toml` — crate manifest and dependency list.

This is a student project created for experimenting with GPU rendering concepts. The code intentionally favors clarity and learning over production hardening. Take ideas freely, but don't treat this as a polished library — it's for study and inspiration.

## High-level overview — how it works

- The app is a single binary in `engine/`. When run it creates a `winit` window and initializes `wgpu`.
- A `State` struct (in `engine/src/main.rs`) owns the `wgpu::Device`, `Queue`, `Surface`, render pipeline, and GPU buffers. `State::new` performs the synchronous setup (using `pollster::block_on` where needed).
- Vertex types use `bytemuck` to cast Rust slices safely into GPU buffers. Shaders are stored in `engine/src/shaders/` and loaded with `include_str!("shaders/<file>.wgsl")` so they are compiled into the binary.
- The render loop is driven by `winit` events. The pipeline expects WGSL entry point names like `vs_main` and `fs_main` (match the existing shaders or update pipeline creation accordingly).

Key files to inspect when learning:

- `engine/src/main.rs` — lifecycle, pipeline creation, vertex/index buffer setup.
- `engine/src/shaders/shader.wgsl` — example vertex + fragment shader in WGSL.
- `engine/Cargo.toml` — dependencies (`wgpu`, `winit`, `bytemuck`, `pollster`, `env_logger`, `anyhow`).

## Build & run (Windows PowerShell)

Open PowerShell at the repository root (or inside the `engine/` folder) and run:

```powershell
# From repo root
cargo run --manifest-path engine/Cargo.toml

# If you want logs (info/debug)
$env:RUST_LOG = "info"; cargo run --manifest-path engine/Cargo.toml
```

Notes:
- The app depends on a working GPU backend (DirectX / Vulkan / Metal depending on platform). On Windows make sure GPU drivers are up to date.
- `wgpu` selects a surface format from the adapter capabilities — be cautious if you modify color formats.

## Development notes and conventions

- Shaders are placed in `engine/src/shaders/` and referenced via `include_str!(...)`. Add new WGSL files there and update pipeline entry points if necessary.
- Vertex data types derive `bytemuck::Pod` + `Zeroable` and are transferred to GPU via `bytemuck::cast_slice(...)`. Keep `Vertex::desc()` in sync with WGSL `@location` attributes.
- `State::new` currently uses `pollster::block_on` — async wiring is intentionally simple for learning.
- Error handling in the project is minimal and may use `.unwrap()` in some places — that's a deliberate tradeoff for a teaching project.

## Repository housekeeping

- Ignore build artifacts — add a `.gitignore` with `/target/` and editor/OS files.
- For binary projects like this one, keeping `engine/Cargo.lock` committed is recommended for reproducible builds. If you prefer not to track it, add it to `.gitignore` (but note the consequences).

## License & use

This repository is a student test project and provided as-is for educational purposes. You may inspect the source and use ideas for learning and personal projects, but:

- Do not use this repository as-is in production systems.
- Do not claim this project as your own work if you reuse large portions; provide attribution where appropriate.
- If secrets or credentials are discovered here (shouldn't be), treat them as compromised and remove/rotate them.

If you want a license header or an official license file added, say which license you prefer (MIT, Apache-2.0, etc.) and I can add it.

## Quick checklist for pushing to GitHub

1. Add a `.gitignore` at repo root ignoring `/target/`, editor files, OS files and secrets.
2. Ensure you committed only source files and `engine/Cargo.lock` (if you want reproducible builds).
3. Run `git status` to verify no large build artifacts are tracked.
4. Create the repository on GitHub and push your `main` branch.
