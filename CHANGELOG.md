<!-- Caminho relativo: CHANGELOG.md -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project aims to follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
- Unit tests for CAD file helper services and updated E2E coverage for the Phase 2 interface.

### Changed

- Root `README.md` updated from Phase 1 scaffold status to Phase 2 viewer integration status.
- Development and architecture docs updated to reflect the concrete decision to integrate the MLightCAD ecosystem through `@mlightcad/cad-simple-viewer` in Svelte.
- App metadata and landing page replaced by an initial working viewer workspace.

### Documented

- Phase 0 planning artifacts for architecture, API contracts, changelog policy, and development workflow.
- Decision to keep NeoCAD under MIT with voluntary donations.
- Initial product scope: Windows and Linux desktop targets with SvelteKit + Tauri 2.
- MVP direction: visualization plus basic editing from the beginning.
