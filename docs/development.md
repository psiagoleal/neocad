<!-- Caminho relativo: docs/development.md -->

# Guia de desenvolvimento

## Objetivo deste documento

Este guia descreve como preparar o ambiente, validar a integração atual do viewer e continuar a evolução do NeoCAD após o início da Fase 2.

## Fase atual

O projeto está em **Fase 2 — integração inicial do viewer**. O repositório agora contém:

- frontend SvelteKit funcional com TypeScript;
- modo SPA configurado com `@sveltejs/adapter-static` e fallback `index.html`;
- shell desktop Tauri 2 inicializado em `src-tauri/`;
- integração inicial com o ecossistema MLightCAD usando `@mlightcad/cad-simple-viewer`;
- abertura de arquivos `DWG` e `DXF` por diálogo nativo no Tauri;
- fallback de abertura por input local quando a aplicação roda em navegador;
- workers estáticos vendorizados em `static/workers/` para DXF, DWG e MTEXT;
- lista de recentes com persistência em `AppConfig` no runtime Tauri e arrastar-e-soltar na interface da Fase 2;
- `Makefile` e fluxo CMake para tarefas frequentes e cross-build Windows inicial;
- lint, formatação e testes básicos configurados.

## Stack validada

- **SvelteKit** como frontend principal
- **Svelte 5** para UI
- **Tauri 2** como shell desktop
- **TypeScript** para a aplicação web
- **Rust** para integração nativa
- **pnpm** como gerenciador de pacotes JavaScript
- **`@mlightcad/cad-simple-viewer`** como núcleo framework-agnostic do viewer CAD

## Referências oficiais consideradas

### SvelteKit

A documentação oficial do SvelteKit indica o uso do CLI `sv` para criação de novos projetos.

Comando utilizado como referência de scaffold:

- `npx sv create`

### Tauri 2

A documentação oficial do Tauri 2 recomenda, para SvelteKit:

- uso de `adapter-static`;
- configuração de `frontendDist` para `build/`;
- preferência por modo SPA quando a aplicação depende de APIs disponíveis apenas no ambiente WebView.

NeoCAD segue essa orientação com:

- `@sveltejs/adapter-static` configurado com `fallback: 'index.html'`;
- `src/routes/+layout.ts` com `export const ssr = false;`;
- `src-tauri/tauri.conf.json` apontando para `../build`.

### Tauri plugins

Para a abertura local de desenhos, a implementação atual usa:

- `@tauri-apps/plugin-dialog`
- `@tauri-apps/plugin-fs`
- `tauri-plugin-dialog`
- `tauri-plugin-fs`

### Workers do viewer

O runtime do viewer depende de workers separados para:

- parser DXF;
- parser DWG;
- renderização de MTEXT.

Para evitar falhas de abertura em builds desktop, esses workers foram copiados para `static/workers/` e o adaptador do NeoCAD aponta explicitamente para eles.

## Decisão de integração com o upstream

Embora o projeto continue sendo um wrapper para o ecossistema `cad-viewer`, a integração Svelte foi iniciada por meio de **`@mlightcad/cad-simple-viewer`**.

Essa escolha foi feita porque:

- `@mlightcad/cad-viewer` é um componente Vue 3 pronto, com UI própria;
- NeoCAD usa Svelte/SvelteKit e precisa controlar sua própria interface;
- `@mlightcad/cad-simple-viewer` fornece o núcleo de documentos, comandos, renderização e eventos sem acoplamento a Vue.

Em outras palavras: no NeoCAD, o pacote `cad-simple-viewer` funciona como a camada de núcleo do upstream, enquanto a interface fica sob responsabilidade do app Svelte.

## Pré-requisitos

### Node.js e pnpm

- Instalar `Node.js` LTS.
- Executar `corepack enable` para habilitar `pnpm`.
- Confirmar versões com `node -v` e `pnpm -v`.

### Rust

- Instalar `rustup`.
- Usar toolchain estável.
- No Windows, preferir `stable-msvc`.

### Dependências de sistema do Tauri 2

#### Windows

Segundo a documentação oficial do Tauri 2:

- instalar **Microsoft C++ Build Tools** com o workload “Desktop development with C++”;
- garantir a presença do **Microsoft Edge WebView2 Runtime**;
- instalar Rust com toolchain MSVC.

#### Linux

Para Ubuntu/Debian, a documentação oficial do Tauri 2 recomenda dependências como:

- `libwebkit2gtk-4.1-dev`
- `build-essential`
- `curl`
- `wget`
- `file`
- `libxdo-dev`
- `libssl-dev`
- `libayatana-appindicator3-dev`
- `librsvg2-dev`

## Fluxo atual de desenvolvimento

### Instalar dependências

```bash
pnpm install
```

### Validar o frontend

```bash
pnpm check
pnpm lint
pnpm test
pnpm build
```

### Validar o backend desktop

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

### Atalhos com Makefile

```bash
make help
make check
make lint
make test
make tauri-dev
```

### Fluxo CMake

```bash
cmake --preset linux-default
cmake --build --preset smoke
```

Para detalhes do build Windows x64 via NSIS, consulte `docs/windows-cross-build.md`.

### Executar o shell desktop

```bash
pnpm tauri dev
```

### Gerar build desktop

```bash
pnpm tauri build
```

## Testes

### Testes unitários

Os testes unitários usam `Vitest`.

```bash
pnpm test
```

### Testes E2E

Os testes E2E usam `Playwright`. Antes da primeira execução, instale os browsers:

```bash
pnpm exec playwright install
pnpm test:e2e
```

## Convenções atuais

### Organização de código

- componentes reutilizáveis em `src/lib/components`;
- serviços em `src/lib/services`;
- adaptador do viewer em `src/lib/viewer`;
- tipos compartilhados em `src/lib/types`;
- comandos nativos e plugins Tauri em `src-tauri/src`.

### Formatação e qualidade

Ferramentas configuradas ou previstas:

- `prettier` para frontend;
- `eslint` para TypeScript/Svelte;
- `rustfmt` para Rust;
- `cargo clippy` para lint nativo.

### Testes

Estratégia atual:

- testes unitários para contratos básicos do frontend e helpers de arquivos CAD;
- base E2E pronta para cobrir a tela principal e ações iniciais;
- expansão futura para fluxos desktop integrados, drag-and-drop e recentes.

## Próxima meta técnica

A próxima etapa prática é **aprofundar a Fase 2**, com foco em:

1. estruturar painéis de propriedades e camadas;
2. expor melhor os comandos de edição básica na UI;
3. persistir preferências do usuário e ampliar a reabertura entre sessões com mais robustez;
4. estudar exportação e persistência local complementar para etapas seguintes do MVP.
