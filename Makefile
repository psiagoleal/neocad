# Caminho relativo: Makefile

.PHONY: help install check lint format test build cargo-check tauri-dev tauri-debug tauri-debug-nobundle tauri-build e2e-install e2e cmake-configure cmake-smoke cmake-linux-bundle cmake-windows-x64-nsis clean

help:
	@echo "Comandos disponíveis:"
	@echo "  make install                  - instala dependências JavaScript"
	@echo "  make check                    - roda svelte-check"
	@echo "  make lint                     - roda prettier --check e eslint"
	@echo "  make format                   - formata o projeto"
	@echo "  make test                     - roda os testes unitários"
	@echo "  make build                    - gera build web do SvelteKit"
	@echo "  make cargo-check              - valida o backend Tauri/Rust"
	@echo "  make tauri-dev                - inicia o app desktop em modo dev"
	@echo "  make tauri-debug-nobundle     - gera binário debug sem bundle"
	@echo "  make tauri-build              - gera build desktop padrão"
	@echo "  make e2e-install              - instala o Chromium do Playwright"
	@echo "  make e2e                      - roda os testes E2E"
	@echo "  make cmake-configure          - configura o build CMake"
	@echo "  make cmake-smoke             - roda smoke checks via CMake"
	@echo "  make cmake-linux-bundle       - gera bundle Linux via CMake"
	@echo "  make cmake-windows-x64-nsis   - tenta cross-build Windows NSIS via CMake"
	@echo "  make clean                    - remove artefatos web e CMake"

install:
	pnpm install

check:
	pnpm check

lint:
	pnpm lint

format:
	pnpm format

test:
	pnpm test

build:
	pnpm build

cargo-check:
	cargo check --manifest-path src-tauri/Cargo.toml

tauri-dev:
	pnpm tauri dev

tauri-debug:
	pnpm tauri build --debug

tauri-debug-nobundle:
	pnpm tauri build --debug --no-bundle

tauri-build:
	pnpm tauri build

e2e-install:
	pnpm exec playwright install chromium

e2e:
	pnpm test:e2e

cmake-configure:
	cmake --preset linux-default

cmake-smoke: cmake-configure
	cmake --build --preset smoke

cmake-linux-bundle: cmake-configure
	cmake --build --preset linux-bundle

cmake-windows-x64-nsis: cmake-configure
	cmake --build --preset windows-x64-nsis

clean:
	rm -rf build .svelte-kit
	rm -rf build/cmake
