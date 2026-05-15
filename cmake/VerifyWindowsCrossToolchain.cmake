cmake_minimum_required(VERSION 3.24)

if(NOT DEFINED NEOCAD_WINDOWS_TARGET OR NEOCAD_WINDOWS_TARGET STREQUAL "")
	set(NEOCAD_WINDOWS_TARGET "x86_64-pc-windows-msvc")
endif()

string(REPLACE "-" "_" NEOCAD_WINDOWS_TARGET_ENV_SAFE "${NEOCAD_WINDOWS_TARGET}")

if(NOT DEFINED CARGO_XWIN_EXECUTABLE OR CARGO_XWIN_EXECUTABLE STREQUAL "" OR CARGO_XWIN_EXECUTABLE MATCHES "-NOTFOUND$")
	message(FATAL_ERROR
		"cargo-xwin nao foi encontrado no PATH. Instale com `cargo install --locked cargo-xwin` antes de rodar os targets Windows."
	)
endif()

if(NOT EXISTS "${CARGO_XWIN_EXECUTABLE}")
	message(FATAL_ERROR
		"cargo-xwin foi configurado, mas o executavel nao existe: ${CARGO_XWIN_EXECUTABLE}"
	)
endif()

if(NOT DEFINED LLVM_RC_EXECUTABLE OR LLVM_RC_EXECUTABLE STREQUAL "" OR LLVM_RC_EXECUTABLE MATCHES "-NOTFOUND$")
	message(FATAL_ERROR
		"llvm-rc nao foi encontrado. Para cross-build Windows com Tauri no target ${NEOCAD_WINDOWS_TARGET}, instale LLVM no host e garanta que `llvm-rc` esteja disponivel no PATH.\n\n"
		"Exemplo em Ubuntu/Debian: `sudo apt install llvm lld clang`\n"
		"Se sua distribuicao instalar apenas uma variante versionada, exporte `RC_${NEOCAD_WINDOWS_TARGET_ENV_SAFE}` apontando para ela ou rode o target depois de adicionar o binario ao PATH."
	)
endif()

if(NOT EXISTS "${LLVM_RC_EXECUTABLE}")
	message(FATAL_ERROR
		"llvm-rc foi configurado, mas o executavel nao existe: ${LLVM_RC_EXECUTABLE}"
	)
endif()

message(STATUS "Windows cross-build prerequisites detected:")
message(STATUS "  target triple : ${NEOCAD_WINDOWS_TARGET}")
message(STATUS "  cargo-xwin    : ${CARGO_XWIN_EXECUTABLE}")
message(STATUS "  llvm-rc       : ${LLVM_RC_EXECUTABLE}")
