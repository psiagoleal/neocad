.PHONY: help install check lint format test build kernel-wasm kernel-wasm-dev workers-sync workers-check licenses-check licenses-list cargo-check kernel-fmt kernel-clippy kernel-test kernel-cargo-check kernel-check tauri-dev tauri-debug tauri-debug-nobundle tauri-build e2e-install e2e cmake-configure cmake-smoke cmake-linux-bundle cmake-windows-x64-portable cmake-windows-x64-portable-fixed-runtime cmake-windows-x64-nsis cmake-windows-x64-nsis-fixed-runtime release-version release-tag release-run release-watch release-assets release-publish dist-test dist-test-linux dist-test-windows dist-test-deps clean

help:
	@echo "Comandos disponíveis:"
	@echo "  make install                           - instala dependências JavaScript"
	@echo "  make check                             - roda svelte-check"
	@echo "  make lint                              - roda prettier --check e eslint"
	@echo "  make format                            - formata o projeto"
	@echo "  make test                              - roda os testes unitários"
	@echo "  make build                             - gera build web do SvelteKit"
	@echo "  make kernel-wasm                       - compila o kernel CAD para WebAssembly (release)"
	@echo "  make kernel-wasm-dev                   - idem, perfil de desenvolvimento (mais rápido)"
	@echo "  make workers-sync                      - copia os workers do upstream de node_modules"
	@echo "  make workers-check                     - falha se os workers divergirem do manifesto"
	@echo "  make licenses-check                    - valida licenças de runtime contra a política"
	@echo "  make licenses-list                     - lista o inventário de licenças de runtime"
	@echo "  make cargo-check                       - valida o backend Tauri/Rust"
	@echo "  make kernel-check                      - fmt + check + clippy + test do kernel CAD"
	@echo "  make kernel-fmt                        - confere a formatação do kernel CAD"
	@echo "  make kernel-clippy                     - roda o clippy do kernel CAD"
	@echo "  make kernel-test                       - roda os testes do kernel CAD"
	@echo "  make tauri-dev                         - inicia o app desktop em modo dev"
	@echo "  make tauri-debug-nobundle              - gera binário debug sem bundle"
	@echo "  make tauri-build                       - gera build desktop padrão"
	@echo "  make e2e-install                       - instala o Chromium do Playwright"
	@echo "  make e2e                               - roda os testes E2E"
	@echo "  make cmake-configure                   - configura o build CMake"
	@echo "  make cmake-smoke                       - roda smoke checks via CMake"
	@echo "  make cmake-linux-bundle                - gera bundle Linux via CMake"
	@echo "  make cmake-windows-x64-portable        - gera .zip Windows portátil; usa Fixed Runtime se ele existir em .webview2/fixed-runtime-x64"
	@echo "  make cmake-windows-x64-portable-fixed-runtime - gera .zip Windows portátil exigindo Fixed Runtime local"
	@echo "  make cmake-windows-x64-nsis            - gera instalador NSIS current-user com embedBootstrapper"
	@echo "  make cmake-windows-x64-nsis-fixed-runtime - gera instalador NSIS current-user com Fixed Runtime local"
	@echo "  make dist-test                         - builds locais de TESTE: Linux + Windows"
	@echo "  make dist-test-linux                   - apenas o build de teste Linux"
	@echo "  make dist-test-windows                 - apenas o build de teste Windows (cross MinGW)"
	@echo "  make dist-test-deps                    - diagnostica pré-requisitos dos builds de teste"
	@echo "  Release (empacota na CI; ver .github/workflows/release.yml):"
	@echo "  make release-version                   - imprime a versão atual (package.json)"
	@echo "  make release-tag                       - cria a tag v<versão>, faz push e dispara a pipeline"
	@echo "  make release-run                       - reempacota uma tag já existente (TAG=v0.2.0)"
	@echo "  make release-watch                     - acompanha a última execução da pipeline"
	@echo "  make release-assets                    - lista os artefatos da release da versão atual"
	@echo "  make release-publish                   - tira a release do rascunho (revise os binários antes)"
	@echo "  make clean                             - remove artefatos web e CMake"

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

kernel-wasm:
	pnpm kernel:build

kernel-wasm-dev:
	pnpm kernel:build:dev

workers-sync:
	pnpm workers:sync

workers-check:
	pnpm workers:check

licenses-check:
	pnpm licenses:check

licenses-list:
	@pnpm licenses:list

cargo-check:
	cargo check --manifest-path src-tauri/Cargo.toml

# Os alvos do kernel entram em `kernel/` em vez de usar `--manifest-path`, porque
# o `rust-toolchain.toml` só é aplicado a partir do diretório de trabalho.
kernel-fmt:
	cd kernel && cargo fmt --all --check

kernel-clippy:
	cd kernel && cargo clippy --all-targets -- -D warnings

kernel-test:
	cd kernel && cargo test --all

kernel-cargo-check:
	cd kernel && cargo check --all-targets

# Mesma sequência do job `kernel` da CI: fmt -> check -> clippy -> test.
kernel-check: kernel-fmt kernel-cargo-check kernel-clippy kernel-test
	@echo "kernel: fmt, check, clippy e testes OK"

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

cmake-windows-x64-portable: cmake-configure
	cmake --build --preset windows-x64-portable

cmake-windows-x64-portable-fixed-runtime: cmake-configure
	cmake --build --preset windows-x64-portable-fixed-runtime

cmake-windows-x64-nsis: cmake-configure
	cmake --build --preset windows-x64-nsis

cmake-windows-x64-nsis-fixed-runtime: cmake-configure
	cmake --build --preset windows-x64-nsis-fixed-runtime

# Builds locais de teste. Sempre os dois alvos; ver scripts/build-test.sh.
dist-test:
	./scripts/build-test.sh all

dist-test-linux:
	./scripts/build-test.sh linux

dist-test-windows:
	./scripts/build-test.sh windows

dist-test-deps:
	@./scripts/build-test.sh deps

# O empacotamento saiu daqui: quem compila Linux e Windows é
# `.github/workflows/release.yml`, e o Windows sai de runner Windows com MSVC
# nativo. O que resta na máquina local é disparar, acompanhar e aprovar.
release-version:
	@./scripts/release.sh version

# Empurrar a tag é o gatilho da pipeline; não há mais build local a fazer.
release-tag:
	./scripts/release.sh tag

# Reempacota uma tag que já existe, sem criar tag nova. Útil quando a mudança
# foi na própria pipeline, e não no produto.
release-run:
	@tag="$${TAG:-v$$(./scripts/release.sh version)}"; \
	echo "Disparando a pipeline de release para $$tag"; \
	gh workflow run release.yml -f tag="$$tag"

release-watch:
	@gh run watch "$$(gh run list --workflow=release.yml --limit 1 --json databaseId -q '.[0].databaseId')"

release-assets:
	@tag="$${TAG:-v$$(./scripts/release.sh version)}"; \
	gh release view "$$tag" --json isDraft,assets \
	  -q '"rascunho=\(.isDraft)", (.assets[] | "\(.name)  \(.size/1048576 | floor) MB")'

# A pipeline cria a release como rascunho de propósito. Publicar é um ato
# deliberado de quem conferiu os binários, e por isso mora aqui e não lá.
release-publish:
	@tag="$${TAG:-v$$(./scripts/release.sh version)}"; \
	gh release edit "$$tag" --draft=false --latest

clean:
	rm -rf build .svelte-kit
	rm -rf .cmake
	rm -rf src/lib/kernel/pkg
