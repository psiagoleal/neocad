cmake_minimum_required(VERSION 3.24)
project(NeoCAD LANGUAGES NONE)

set(NEOCAD_ROOT_DIR "${CMAKE_CURRENT_SOURCE_DIR}")
set(NEOCAD_WINDOWS_TARGET "x86_64-pc-windows-msvc" CACHE STRING "Rust target triple used for Windows builds")
string(REPLACE "-" "_" NEOCAD_WINDOWS_TARGET_ENV_SAFE "${NEOCAD_WINDOWS_TARGET}")
set(
	NEOCAD_WINDOWS_FIXED_RUNTIME_DIR
	"${NEOCAD_ROOT_DIR}/.webview2/fixed-runtime-x64"
	CACHE PATH
	"Directory containing an extracted WebView2 Fixed Runtime"
)
set(
	NEOCAD_WINDOWS_PORTABLE_STAGE_DIR
	"${NEOCAD_ROOT_DIR}/build/windows/portable/NeoCAD-portable-x64"
	CACHE PATH
	"Directory used to stage the Windows portable package"
)
set(
	NEOCAD_WINDOWS_PORTABLE_ZIP
	"${NEOCAD_ROOT_DIR}/build/windows/portable/NeoCAD-portable-x64.zip"
	CACHE FILEPATH
	"Zip file created for the Windows portable package"
)

find_program(PNPM_EXECUTABLE pnpm REQUIRED)
find_program(CARGO_EXECUTABLE cargo REQUIRED)
find_program(CARGO_XWIN_EXECUTABLE cargo-xwin)
find_program(
	LLVM_RC_EXECUTABLE
	NAMES llvm-rc llvm-rc-19 llvm-rc-18 llvm-rc-17 llvm-rc-16 llvm-rc-15 llvm-rc-14 llvm-rc-13
	PATHS
		/usr/lib/llvm-19/bin
		/usr/lib/llvm-18/bin
		/usr/lib/llvm-17/bin
		/usr/lib/llvm-16/bin
		/usr/lib/llvm-15/bin
		/usr/lib/llvm-14/bin
		/usr/lib/llvm-13/bin
)

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
	COMMAND ${CMAKE_COMMAND} -DNEOCAD_WINDOWS_TARGET:STRING=${NEOCAD_WINDOWS_TARGET} -DCARGO_XWIN_EXECUTABLE:FILEPATH=${CARGO_XWIN_EXECUTABLE} -DLLVM_RC_EXECUTABLE:FILEPATH=${LLVM_RC_EXECUTABLE} -P ${CMAKE_CURRENT_SOURCE_DIR}/cmake/VerifyWindowsCrossToolchain.cmake
	COMMAND ${CMAKE_COMMAND} -E env RC_${NEOCAD_WINDOWS_TARGET_ENV_SAFE}=${LLVM_RC_EXECUTABLE} ${PNPM_EXECUTABLE} tauri build --runner cargo-xwin --target ${NEOCAD_WINDOWS_TARGET} --bundles nsis --config src-tauri/tauri.windows.conf.json --no-sign
	WORKING_DIRECTORY ${NEOCAD_ROOT_DIR}
	USES_TERMINAL
	VERBATIM
)

add_custom_target(
	neocad_windows_x64_nsis_fixed_runtime
	COMMAND ${CMAKE_COMMAND} -DNEOCAD_WINDOWS_TARGET:STRING=${NEOCAD_WINDOWS_TARGET} -DCARGO_XWIN_EXECUTABLE:FILEPATH=${CARGO_XWIN_EXECUTABLE} -DLLVM_RC_EXECUTABLE:FILEPATH=${LLVM_RC_EXECUTABLE} -P ${CMAKE_CURRENT_SOURCE_DIR}/cmake/VerifyWindowsCrossToolchain.cmake
	COMMAND ${CMAKE_COMMAND} -DNEOCAD_WINDOWS_FIXED_RUNTIME_DIR:PATH=${NEOCAD_WINDOWS_FIXED_RUNTIME_DIR} -P ${CMAKE_CURRENT_SOURCE_DIR}/cmake/VerifyWindowsFixedRuntime.cmake
	COMMAND ${CMAKE_COMMAND} -E env RC_${NEOCAD_WINDOWS_TARGET_ENV_SAFE}=${LLVM_RC_EXECUTABLE} ${PNPM_EXECUTABLE} tauri build --runner cargo-xwin --target ${NEOCAD_WINDOWS_TARGET} --bundles nsis --config src-tauri/tauri.windows.fixed-runtime.conf.json --no-sign
	WORKING_DIRECTORY ${NEOCAD_ROOT_DIR}
	USES_TERMINAL
	VERBATIM
)

add_custom_target(
	neocad_windows_x64_portable
	COMMAND ${CMAKE_COMMAND} -DNEOCAD_WINDOWS_TARGET:STRING=${NEOCAD_WINDOWS_TARGET} -DCARGO_XWIN_EXECUTABLE:FILEPATH=${CARGO_XWIN_EXECUTABLE} -DLLVM_RC_EXECUTABLE:FILEPATH=${LLVM_RC_EXECUTABLE} -P ${CMAKE_CURRENT_SOURCE_DIR}/cmake/VerifyWindowsCrossToolchain.cmake
	COMMAND ${CMAKE_COMMAND} -E env RC_${NEOCAD_WINDOWS_TARGET_ENV_SAFE}=${LLVM_RC_EXECUTABLE} ${PNPM_EXECUTABLE} tauri build --runner cargo-xwin --target ${NEOCAD_WINDOWS_TARGET} --no-bundle --config src-tauri/tauri.windows.conf.json --no-sign
	COMMAND ${CMAKE_COMMAND} -DNEOCAD_ROOT_DIR:PATH=${NEOCAD_ROOT_DIR} -DNEOCAD_WINDOWS_TARGET:STRING=${NEOCAD_WINDOWS_TARGET} -DNEOCAD_WINDOWS_FIXED_RUNTIME_DIR:PATH=${NEOCAD_WINDOWS_FIXED_RUNTIME_DIR} -DNEOCAD_WINDOWS_PORTABLE_STAGE_DIR:PATH=${NEOCAD_WINDOWS_PORTABLE_STAGE_DIR} -DNEOCAD_WINDOWS_PORTABLE_ZIP:FILEPATH=${NEOCAD_WINDOWS_PORTABLE_ZIP} -DNEOCAD_REQUIRE_FIXED_RUNTIME:BOOL=OFF -P ${CMAKE_CURRENT_SOURCE_DIR}/cmake/StageWindowsPortable.cmake
	WORKING_DIRECTORY ${NEOCAD_ROOT_DIR}
	USES_TERMINAL
	VERBATIM
)

add_custom_target(
	neocad_windows_x64_portable_fixed_runtime
	COMMAND ${CMAKE_COMMAND} -DNEOCAD_WINDOWS_TARGET:STRING=${NEOCAD_WINDOWS_TARGET} -DCARGO_XWIN_EXECUTABLE:FILEPATH=${CARGO_XWIN_EXECUTABLE} -DLLVM_RC_EXECUTABLE:FILEPATH=${LLVM_RC_EXECUTABLE} -P ${CMAKE_CURRENT_SOURCE_DIR}/cmake/VerifyWindowsCrossToolchain.cmake
	COMMAND ${CMAKE_COMMAND} -E env RC_${NEOCAD_WINDOWS_TARGET_ENV_SAFE}=${LLVM_RC_EXECUTABLE} ${PNPM_EXECUTABLE} tauri build --runner cargo-xwin --target ${NEOCAD_WINDOWS_TARGET} --no-bundle --config src-tauri/tauri.windows.conf.json --no-sign
	COMMAND ${CMAKE_COMMAND} -DNEOCAD_ROOT_DIR:PATH=${NEOCAD_ROOT_DIR} -DNEOCAD_WINDOWS_TARGET:STRING=${NEOCAD_WINDOWS_TARGET} -DNEOCAD_WINDOWS_FIXED_RUNTIME_DIR:PATH=${NEOCAD_WINDOWS_FIXED_RUNTIME_DIR} -DNEOCAD_WINDOWS_PORTABLE_STAGE_DIR:PATH=${NEOCAD_WINDOWS_PORTABLE_STAGE_DIR} -DNEOCAD_WINDOWS_PORTABLE_ZIP:FILEPATH=${NEOCAD_WINDOWS_PORTABLE_ZIP} -DNEOCAD_REQUIRE_FIXED_RUNTIME:BOOL=ON -P ${CMAKE_CURRENT_SOURCE_DIR}/cmake/StageWindowsPortable.cmake
	WORKING_DIRECTORY ${NEOCAD_ROOT_DIR}
	USES_TERMINAL
	VERBATIM
)

add_custom_target(neocad_smoke ALL DEPENDS neocad_check neocad_lint neocad_test neocad_cargo_check)
