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
- Initial NeoCAD landing page, shared frontend app metadata module, and unit/E2E test scaffolding.
- Root package scripts for development, validation, testing, and Tauri commands.

### Changed

- Root `README.md` updated to reflect the real scaffolded state of the repository.
- Development guide updated from planning-only instructions to the implemented Phase 1 workflow.

### Documented

- Phase 0 planning artifacts for architecture, API contracts, changelog policy, and development workflow.
- Decision to keep NeoCAD under MIT with voluntary donations.
- Initial product scope: Windows and Linux desktop targets with SvelteKit + Tauri 2.
- MVP direction: visualization plus basic editing from the beginning.
