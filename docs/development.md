<!-- Caminho relativo: docs/development.md -->

# Guia de desenvolvimento

## Objetivo deste documento

Este guia descreve como preparar o ambiente, validar o scaffold atual e continuar a evolução do NeoCAD após a conclusão da Fase 1.

## Fase atual

A Fase 1 do projeto está concluída. O repositório agora contém:

- frontend SvelteKit funcional com TypeScript;
- modo SPA configurado com `@sveltejs/adapter-static` e fallback `index.html`;
- shell desktop Tauri 2 inicializado em `src-tauri/`;
- lint, formatação e testes básicos configurados.

## Stack validada

- **SvelteKit** como frontend principal
- **Svelte 5** para UI
- **Tauri 2** como shell desktop
- **TypeScript** para a aplicação web
- **Rust** para integração nativa
- **pnpm** como gerenciador de pacotes JavaScript

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
pnpm test
pnpm build
```

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

## Convenções iniciais

### Organização de código

- componentes reutilizáveis em `src/lib/components`;
- serviços em `src/lib/services`;
- adaptador do `cad-viewer` em `src/lib/viewer`;
- stores em `src/lib/stores`;
- comandos nativos organizados por domínio em `src-tauri/src/commands`.

### Formatação e qualidade

Ferramentas configuradas ou previstas:

- `prettier` para frontend;
- `eslint` para TypeScript/Svelte;
- `rustfmt` para Rust;
- `cargo clippy` para lint nativo.

### Testes

Estratégia atual:

- testes unitários para contratos básicos do frontend;
- base E2E pronta para cobrir telas principais;
- expansão futura para fluxos desktop integrados e abertura de arquivos.

## Próxima meta técnica

A próxima etapa prática do projeto é a **Fase 2**, com foco em:

1. integrar o `cad-viewer` via uma camada adaptadora própria;
2. abrir arquivos locais `DWG` e `DXF` pelo shell desktop;
3. validar renderização inicial dentro da janela Tauri;
4. preparar o terreno para edição básica.
