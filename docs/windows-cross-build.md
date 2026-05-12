<!-- Caminho relativo: docs/windows-cross-build.md -->

# Build cross-platform para Windows

## Objetivo

Este documento descreve o fluxo inicial de build Windows para NeoCAD a partir de Linux/WSL usando **CMake** como orquestrador.

## Escopo realista

O fluxo atual foi desenhado para:

- **cross-build Windows x64**;
- geração de **instalador NSIS**;
- uso de **CMake** para orquestrar comandos do `pnpm`, `cargo` e `tauri`.

## Limitações importantes

- **MSI/WiX não é suportado a partir de Linux/WSL**.
- O fluxo cross-platform do Tauri 2 para Windows é possível, mas **menos testado** do que builds nativos em Windows.
- Assinatura de binários Windows em cross-build exige ferramenta externa; por isso o target atual usa `--no-sign`.

## Pré-requisitos no Linux/WSL

Segundo a documentação oficial do Tauri 2, o fluxo de cross-build para Windows requer pelo menos:

- `nsis`
- `llvm`
- `lld`
- target Rust `x86_64-pc-windows-msvc`
- `cargo-xwin`

Exemplo de preparação em Ubuntu/Debian:

```bash
sudo apt install nsis llvm lld clang
rustup target add x86_64-pc-windows-msvc
cargo install --locked cargo-xwin
```

## Arquivos adicionados no repositório

- `CMakeLists.txt`
- `CMakePresets.json`
- `cmake/NeoCADTargets.cmake`
- `src-tauri/tauri.windows.conf.json`
- `Makefile`

## Presets disponíveis

### Configurar CMake

```bash
cmake --preset linux-default
```

### Rodar smoke checks

```bash
cmake --build --preset smoke
```

### Gerar bundle Linux

```bash
cmake --build --preset linux-bundle
```

### Tentar build Windows x64 com NSIS

```bash
cmake --build --preset windows-x64-nsis
```

## Target Windows atual

O target `windows-x64-nsis` executa um comando equivalente a:

```bash
pnpm tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc --bundles nsis --config src-tauri/tauri.windows.conf.json --no-sign
```

## Configuração Windows do Tauri

O arquivo `src-tauri/tauri.windows.conf.json` ajusta o build para:

- gerar somente `nsis`;
- usar `embedBootstrapper` para WebView2;
- preparar associações de arquivo para `DWG` e `DXF`.

## Recomendação de release

Mesmo com esse fluxo disponível, a recomendação continua sendo:

- usar **Linux/WSL + CMake** para validação e builds cross-platform iniciais;
- usar **runner Windows nativo** em CI para releases oficiais, assinatura e cenários MSI.
