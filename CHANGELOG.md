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

- ADR 0002 recording the relicensing to GPL-3.0-or-later, its rationale and the discarded alternatives.
- ADR 0003 establishing the in-house, reusable CAD kernel track: a standalone Rust crate workspace under `kernel/`, compiled to WebAssembly for the frontend and linkable natively into the Tauri backend, with the `@mlightcad` upstream demoted to a replaceable parser and renderer. Staged K1 (document model and transactional undo/redo) through K9 (solid modelling with STEP/IGES), covering 2D and 3D B-rep. Copyleft dependencies are confined to the `neocad-io` crate so the kernel can be licensed independently for reuse in other projects.
- K1 broken down into 17 micro-tickets in `docs/tickets/k1-modelo-documento-transacoes.md`, with per-ticket file scope, acceptance criteria, dependency order and known risks.
- `scripts/build-test.sh` and `make dist-test` producing local test builds for both Linux (native, with `.deb`) and Windows x64 in a single run, with prerequisite diagnostics (`make dist-test-deps`) and a generated per-platform `LEIA-ME.txt`. The Windows side cross-builds through the MinGW toolchain, which needs neither `cargo-xwin`, `llvm-rc` nor administrative privileges — suitable for testing, not for distribution.
- ADR 0004 establishing a headless interface for AI agents — `neocad-cli` as the functional core and `neocad-mcp` as a thin Model Context Protocol facade — with mandatory safeguards for automated editing (never overwrite the input file without an explicit flag, `--dry-run` on every edit command, all mutations through the transactional command stack, deterministic output). Recorded as Frente 5 in the roadmap, and extending phase K2 to cover native DXF _reading_, which ADR 0003 had not scheduled because it assumed the browser-based upstream parser.
- DXF construct fixtures derived from bisecting a drawing that would not open (`e2e/dxf-constructs.e2e.ts`). Old-style `POLYLINE`/`VERTEX` and block references open and are counted as not-yet-modelled, without preventing the drawing from loading. A block definition **containing entities** is not readable by the upstream DXF parser at all — recorded as a known failure with `test.fail()`, so the suite breaks the day it starts working and the assertion can be replaced. Since a block with content is how every symbol and title block is defined, DXF files of real origin do not open today; the native DXF reading of phase K2 replaces that parser. Fixtures are synthetic.
- Architecture and API documentation rewritten for the kernel era (MT-K1-17). `docs/architecture.md` now describes the kernel as the source of truth about the drawing, with a layer diagram covering the six crates, an opening flow showing the kernel load happening after the upstream activates the document, and risks restated around the real ones — the scale of the kernel effort, the two live representations coexisting until K5/K6, and compatibility with real-world files. `docs/api.md` documents the document contracts, the single integration boundary and the kernel invariants. Phase K1 is complete: the project has an in-house document model in Rust with every drawing mutation going through a reversible transaction.
- End-to-end coverage of the drawing reaching the in-house model (MT-K1-16): opening `minimal.dxf` reports exactly the two entities and single layer the file contains, a new `with-unsupported.dxf` fixture confirms that an entity the kernel does not model yet is reported without preventing the drawing from opening, the `Editar` menu shows both actions disabled after a load, and the canvas still has real height so the kernel's arrival cannot silently regress what already worked. E2E suite grew from three tests to seven.
- `Editar` menu with `Desfazer` and `Refazer` wired to the kernel command stack (MT-K1-15). Both entries are disabled according to the real stack state and their labels name the action being undone. The route creates the kernel document, loads the drawing into it when the upstream activates a document, and re-reads the history from the kernel after every action rather than keeping a derived copy in sync by hand. Opening a drawing now reports how many entities and layers the kernel received, and how many entities it does not model yet.
- Population of the in-house model from the upstream-opened document (MT-K1-14). `buildDocumentSnapshot` reads the upstream layer table and model space and produces a snapshot, which `CadSession::load` turns into a kernel document in a single bridge crossing, resetting the undo history. Geometry is recognised by the shape of the upstream object rather than by its `type` string, whose values the upstream typings declare nowhere — arcs are matched before circles, since both carry a centre and a radius. Entities the kernel does not model yet are recorded as unsupported and counted instead of aborting the load, so a real drawing still opens.
- NeoCAD document contracts and the kernel boundary service (MT-K1-13): `CadLayer`, `CadEntity`, `CadGeometry`, `CadBounds`, `CadColor` and `CadHistoryState` in `src/lib/types/cad.ts`, with branded `CadLayerId` and `CadEntityId` so the compiler refuses to swap one for the other, mirroring the distinction the kernel makes in Rust. `src/lib/services/cad-document.ts` is the single access point to the WebAssembly kernel, loading it through a dynamic import so the `.wasm` stays out of the initial bundle. Conversion is done by pure functions that validate the kernel's output and fail loudly with the offending field path, since a malformed shape there is a kernel defect rather than untrusted input.
- WebAssembly build wired into the frontend pipeline (MT-K1-12): `scripts/build-kernel.mjs` emits the package into `src/lib/kernel/pkg` and is chained into `pnpm dev` (dev profile) and `pnpm build` (release), following the same derived-not-versioned pattern as the upstream worker sync. It reports prerequisites with actionable install commands, prints the `.wasm` size on every build, and skips recompiling when no kernel source changed — without which every `pnpm build`, including the one Playwright triggers before the E2E suite, would rebuild the whole kernel. CI gained a WebAssembly toolchain step in both the `frontend` and `e2e` jobs.
- WebAssembly facade in `neocad-wasm` (MT-K1-11): a `CadSession` bundling a document with its command stack and exposing layer and entity listings, the model-space bounding box, drawing a line, erasing an entity, toggling a layer, `undo`/`redo` and the history state. Built with `wasm-pack --target web`; the resulting `.wasm` is 107 KB. Identifiers cross the bridge as decimal strings rather than numbers, since a `u64` would surface as a `BigInt` in JavaScript for no benefit — the identifier is opaque either way. All logic sits in a JavaScript-free inner layer so the whole facade, error paths included, is testable with `cargo test` on the host.
- Drawing mutation closed behind recorded edits (MT-K1-10). `Document::edit()` returns a `DocumentEditor` that is the only public way to change entities or layer properties, and it records the inverse of every operation, so no change can happen without producing what undoes it. The direct paths became crate-private and the three that allowed unrecorded mutation (`entity_mut`, `layer_mut`, `move_entity_to_block`) were removed outright. Four `compile_fail` doctests assert that the restriction is enforced by the compiler rather than merely documented. `CommandStack::edit` makes creating a new entity undoable, which a pre-built transaction could not express since it never allocates identifiers. The `Change` primitive moved from `neocad-transaction` into `neocad-model` because crate-private visibility cannot span crates; `neocad-transaction` re-exports it and remains the owner of `Transaction` and `CommandStack`.
- Named transactions and an undo/redo command stack in `neocad-transaction` (MT-K1-09). A `Transaction` groups changes into one atomic user action: if any change fails, the earlier ones are rolled back and the document is left untouched. `CommandStack` stores, for each applied action, the transaction that undoes it — so undoing is simply applying it, which in turn yields the transaction that redoes it, making undo and redo the same operation in opposite directions. Includes a configurable history limit that drops the oldest entry, redo-branch discarding on a new commit, and empty transactions that never consume an undo step.
- Reversible change journal in `neocad-transaction` (MT-K1-08): `Change` covering entity insertion, removal and replacement plus whole-record layer edits, where `apply` returns the change that undoes it, built from the state observed at application time rather than inferred from the request. Undo therefore restores the same entity identifier and the same position in draw order, not merely an equivalent entity. Required new model primitives for exact restoration (`Arena::insert_at`, `BlockRecord::insert_entity_at`, `Document::restore_entity`) and a semantic `PartialEq` for `Document` that compares observable content while ignoring allocation residue.
- `Document` composing the entity arena with the layer, block and text style tables (MT-K1-07), and owning the invariants that span them: an entity is only accepted if its layer exists, the arena and the owning block's ordered list are kept in sync, a layer still holding entities cannot be removed, and removing a block also removes the entities it contains rather than orphaning them. Symbol tables are exposed read-only; every cross-structure change goes through a document method.
- Block and text style tables in `neocad-model` (MT-K1-06): `BlockTable` with an indestructible `*Model_Space` root block holding an ordered entity list (the order being draw order), and `TextStyleTable` with a protected `Standard` style that resolves effective text height — a fixed-height style overrides the height carried by the entity. Names beginning with `*` are rejected on creation, since that prefix is reserved for format-internal names. Name validation is now shared by all three symbol tables so their rules cannot drift apart.
- 2D drawing entities and bounding boxes (MT-K1-05): `Point2` and `Aabb` in `neocad-geometry`; `Entity` in `neocad-model` carrying layer and colour (`ByLayer`, `ByBlock` or explicit) plus a `Geometry` covering `Line`, `Circle`, `Arc`, `Polyline` and `Text`. Arc bounding boxes include every cardinal angle inside the sweep, so arcs crossing an axis are measured correctly rather than being clipped to their endpoints.
- Layer table in `neocad-model` (MT-K1-04): `LayerTable` with an opaque `LayerId` distinct from `EntityId`, layer records carrying name, ACI or true colour, linetype, line weight and off/frozen/locked state, case-insensitive name uniqueness, an indestructible layer `0`, and deterministic alphabetical iteration. Layers are referenced by identifier rather than by name, so renaming is a local operation instead of a sweep over every entity as the DXF representation would force.
- Generational entity storage in `neocad-model` (MT-K1-03): opaque `EntityId` carrying slot index plus generation, and `Arena<T>` with slot reuse, deterministic iteration by index, and stale-identifier rejection — a removed id resolves to `None` instead of silently reaching whichever value later occupied the slot. Exhausted generations retire a slot rather than reissue an already-handed-out identifier.
- CI job for the kernel workspace (MT-K1-02) running `fmt`, `check`, `clippy` and `test` on Linux and Windows, plus `make kernel-check` reproducing the same sequence locally.
- Kernel Rust workspace under `kernel/` (MT-K1-01) with the six crates defined by ADR 0003 — `neocad-geometry`, `neocad-topology`, `neocad-model`, `neocad-transaction`, `neocad-io` and `neocad-wasm` — independent from the `src-tauri/` workspace, with workspace-wide lints enforcing documented public APIs and denying `unsafe`. Crates are still empty; domain types start at MT-K1-03.
- GitHub Actions CI workflow (`.github/workflows/ci.yml`) covering type check, lint, unit tests, frontend build, Rust `fmt`/`check`/`clippy` on Linux and Windows, Playwright E2E, and secret scanning. Closes the CI gap in the project Definition of Done.
- `THIRD-PARTY-LICENSES.md` documenting the provenance and license of every runtime dependency shipped in the binaries.
- Runtime license policy in `scripts/license-policy.json`, enforced on CI by `pnpm licenses:check`, which fails when a license incompatible with GPL-3.0 enters the dependency tree or when a tracked dependency changes its terms upstream.
- `scripts/sync-workers.mjs` to derive the upstream `@mlightcad` workers from `node_modules` at build time, with a versioned `static/workers/workers.manifest.json` recording source version and SHA-256 per file.

### Changed

- **BREAKING (licensing):** the project is relicensed from MIT to **GPL-3.0-or-later**. Two runtime dependencies in the critical CAD-reading path are GPL-3.0 — `@mlightcad/libredwg-web` (LibreDWG WebAssembly build, DWG path) and `@mlightcad/dxf-json` (DXF path) — which already made the distributed binaries a combined work subject to the GPL-3.0 while the project advertised MIT. The declared license now matches the effective one. See ADR 0002.
- Upstream worker bundles are no longer committed to the repository (~9.7 MB of third-party minified code, including the GPL-3.0 LibreDWG WebAssembly build); they are generated during `pnpm dev` and `pnpm build`.
- `pnpm test` now calls `vitest run` directly for deterministic non-interactive execution.
- README badges replaced with live CI status and release version; static badges that never reflected reality were removed.
- `docs/cad-panels-commands-simulation-roadmap.md` realigned to ADR 0003: the panels, commands and FEM/CFD tracks are now sequenced against the kernel phases instead of against the upstream feature set. The missing upstream commands (`UNDO/REDO`, `SCALE`, `MIRROR`, `ARRAY`, `OFFSET`, `TRIM/EXTEND`, `BLOCK/INSERT`) become the kernel specification rather than a ceiling, and the simulation track is made formally dependent on the B-rep phase.

## [0.1.1] - 2026-06-03

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
