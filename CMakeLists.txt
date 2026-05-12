# Caminho relativo: CMakeLists.txt

cmake_minimum_required(VERSION 3.24)
project(NeoCAD LANGUAGES NONE)

set(NEOCAD_ROOT_DIR "${CMAKE_CURRENT_SOURCE_DIR}")

find_program(PNPM_EXECUTABLE pnpm REQUIRED)
find_program(CARGO_EXECUTABLE cargo REQUIRED)

include("${CMAKE_CURRENT_SOURCE_DIR}/cmake/NeoCADTargets.cmake")

neocad_add_pnpm_target(neocad_install install)
neocad_add_pnpm_target(neocad_check check)
neocad_add_pnpm_target(neocad_lint lint)
neocad_add_pnpm_target(neocad_test test)
neocad_add_pnpm_target(neocad_web_build build)
neocad_add_pnpm_target(neocad_tauri_dev tauri dev)
neocad_add_pnpm_target(neocad_tauri_debug_nobundle tauri build --debug --no-bundle)
neocad_add_pnpm_target(neocad_tauri_linux_bundle tauri build)

add_custom_target(
	neocad_cargo_check
	COMMAND ${CARGO_EXECUTABLE} check --manifest-path src-tauri/Cargo.toml
	WORKING_DIRECTORY ${NEOCAD_ROOT_DIR}
	USES_TERMINAL
	VERBATIM
)

add_custom_target(
	neocad_windows_x64_nsis
	COMMAND ${PNPM_EXECUTABLE} tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc --bundles nsis --config src-tauri/tauri.windows.conf.json --no-sign
	WORKING_DIRECTORY ${NEOCAD_ROOT_DIR}
	USES_TERMINAL
	VERBATIM
)

add_custom_target(neocad_smoke ALL DEPENDS neocad_check neocad_lint neocad_test neocad_cargo_check)
