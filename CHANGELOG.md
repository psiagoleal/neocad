<!-- Caminho relativo: CHANGELOG.md -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project aims to follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

This is the canonical history of the project. Recommended sections, in priority
order: `Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`, `Security`.

Maintenance rules:

- every relevant functional change updates this file;
- internal-only changes may be grouped when it makes sense;
- significant documentation changes may also be recorded;
- on release, items move from `[Unreleased]` to a dated version.

## [Unreleased]

### Added

- Official Phase 1 scaffold with SvelteKit, TypeScript, ESLint, Prettier, Vitest, and Playwright.
- Tauri 2 desktop shell initialized under `src-tauri/`.
- SPA-compatible SvelteKit configuration for desktop packaging with `@sveltejs/adapter-static`.
- Root package scripts for development, validation, testing, and Tauri commands.
- Initial Phase 2 integration using `@mlightcad/cad-simple-viewer` as the framework-agnostic upstream core.
- CAD file selection service with Tauri dialog/file-system support and browser fallback.
- Viewer adapter with document activation, progress, message, and command forwarding hooks.
- Desktop UI for opening local `DWG`/`DXF` files, fitting view, toggling background, and sending CAD commands.
- Tauri dialog and filesystem plugins enabled for secure local file opening.
- Vendored worker assets under `static/workers/` to stabilize DXF/DWG and MTEXT parsing in desktop builds.
- Recent documents list with persistent desktop storage in `AppConfig/state/recent-documents.json` and drag-and-drop support in the Phase 2 workspace.
- Desktop workspace flow with initial integration screen, dedicated CAD canvas screen, top navigation, and minimizable messages dock.
- Modular workspace components under `src/lib/components/workspace/` for home, viewer, about, menu, recent documents, and messages.
- Centralized frontend styling under `src/lib/styles/`, loaded globally from `src/routes/+layout.svelte`.
- Desktop-like top menu with `Arquivo`, `Exibir`, `Janela` and `Ajuda`, including dropdown actions over the viewer workspace.
- Windows x64 packaging flow for portable `.zip`, NSIS current-user installer, and optional Fixed WebView2 Runtime variants via CMake.
- `Makefile`, `CMakeLists.txt`, `CMakePresets.json` and Windows cross-build documentation.
- Unit tests for CAD file helper services and updated E2E coverage for the Phase 2 interface, including the reference DWG file.
- Runtime CAD command catalog derived from the upstream command stack, surfaced through `Ajuda > Comandos CAD` as a filterable dialog grouped by category.
- `listCommandDescriptors()` on the viewer adapter and a `cad-commands` service that keep all `@mlightcad` access behind the NeoCAD boundary.
- Unit tests for the command catalog builder.

### Changed

- Root `README.md` updated from Phase 1 scaffold status to Phase 2 viewer integration status.
- Development and architecture docs updated to reflect the concrete decision to integrate the MLightCAD ecosystem through `@mlightcad/cad-simple-viewer` in Svelte.
- Phase 2 docs updated to describe persistent recent-document storage in Tauri with browser fallback.
- App metadata and landing page replaced by an initial working viewer workspace.
- Main UI reorganized from a simultaneous multi-panel integration layout into a screen-based desktop workflow closer to CAD usage.
- Top-level workspace hierarchy refined toward a more desktop-like shell, with a more compact header and reduced redundant viewer metadata.
- Home screen simplified to prioritize file opening and recent documents, with less institutional noise in the initial view.
- Viewer command bar from the upstream core remains available inside the canvas as the primary command entry mechanism.
- Consolidated the changelog maintenance policy into the canonical root `CHANGELOG.md` and removed the redundant `docs/changelog.md`.

### Fixed

- Blank CAD canvas after opening a drawing: the viewer surface collapsed to zero height because it landed on the `auto` grid track whenever the progress bar was absent. The viewer frame now uses a flex column so the canvas always fills the available height. Covered by a new E2E regression test with a minimal DXF fixture.

### Documented

- Phase 0 planning artifacts for architecture, API contracts, changelog policy, and development workflow.
- Decision to keep NeoCAD under MIT with voluntary donations.
- Initial product scope: Windows and Linux desktop targets with SvelteKit + Tauri 2.
- MVP direction: visualization plus basic editing from the beginning.
- Frontend workspace refactor plan and its resulting architecture for modular Svelte components and centralized CSS.
- Roadmap for layers/properties panels, CAD command catalog in `Ajuda`, basic CAD commands, and optional FEM/CFD investigation.
- Upstream capabilities spike (`docs/upstream-capabilities-spike.md`) and ADR 0001 fixing the dynamic command catalog and read panels over the `@mlightcad/data-model` API.
- Agent governance scaffold (`AGENTS.md` as single source of truth, `CLAUDE.md`, skills, `.claude/`, `.cursorrules`, `.env.example`).
