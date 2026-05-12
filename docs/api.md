<!-- Caminho relativo: docs/api.md -->

# API e contratos internos

## Objetivo

Este documento registra os contratos internos atuais do NeoCAD na Fase 2. Eles continuam sendo **internos** e sujeitos a evolução.

## Escopo

NeoCAD, no MVP, não terá uma API pública remota. O foco atual é definir contratos internos entre:

- UI SvelteKit;
- serviços de aplicação;
- adaptador do viewer;
- plugins Tauri;
- núcleo CAD fornecido por `@mlightcad/cad-simple-viewer`.

## Contratos implementados ou iniciados

### Serviço de arquivos CAD

Arquivo principal:

- `src/lib/services/cad-file.ts`

Responsabilidades atuais:

- abrir seletor de arquivo no Tauri com diálogo nativo;
- ler bytes do desenho via plugin de filesystem do Tauri;
- oferecer fallback por `input[type=file]` no navegador;
- validar extensões suportadas no MVP (`.dwg` e `.dxf`).

Funções atuais:

- `selectCadDocument()`
- `readCadDocumentFromPath(path)`
- `createCadDocumentPayloadFromFile(file)`
- `extractCadFileName(path)`
- `isSupportedCadFile(fileName)`
- `getCadRuntimeLabel()`

### Serviço de recentes

Arquivo principal:

- `src/lib/services/recent-documents.ts`

Responsabilidades atuais:

- registrar documentos recentes no frontend;
- listar recentes persistidos em `localStorage`;
- limpar a lista atual de recentes.

Funções atuais:

- `listRecentDocuments()`
- `registerRecentDocument(document)`
- `clearRecentDocuments()`

### Adaptador do viewer

Arquivo principal:

- `src/lib/viewer/neocad-viewer.ts`

Responsabilidades atuais:

- carregar dinamicamente `@mlightcad/cad-simple-viewer`;
- criar e destruir a instância do `AcApDocManager`;
- conectar eventos do upstream à UI Svelte;
- abrir documentos a partir de `ArrayBuffer`;
- expor operações básicas de viewport e comandos.

Métodos atuais:

- `mount(container)`
- `openDocument(payload, mode)`
- `zoomToFit()`
- `toggleBackground()`
- `executeCommand(command)`
- `destroy()`

Observação importante:

- o adaptador agora aponta explicitamente para workers estáticos em `static/workers/` para DXF, DWG e MTEXT.

### Estado do documento no frontend

Tipos principais em:

- `src/lib/types/cad.ts`

Contratos atuais:

- `CadDocumentPayload`
- `CadViewerDocumentState`
- `CadViewerProgressState`
- `CadViewerMessage`
- `CadOpenMode`

## Eventos atualmente consumidos do upstream

A integração atual usa o `eventBus` do `cad-simple-viewer` para reagir a:

- `open-file`
- `open-file-progress`
- `message`
- `failed-to-open-file`
- `font-not-found`

## Plugins Tauri atualmente usados

### Dialog

Uso atual:

- seletor nativo para arquivos CAD.

Permissão relevante:

- `dialog:allow-open`

### File system

Uso atual:

- leitura do arquivo selecionado no fluxo desktop.

Permissão relevante:

- `fs:read-files`

## Comandos futuros ainda previstos

Ainda faz sentido manter no radar:

- persistência de recentes;
- preferências do usuário;
- exportações;
- integração de salvamento.

Mas esses contratos ainda não foram implementados como API estável do app.

## Estabilidade

Até a entrega do MVP:

- os contratos aqui descritos devem ser tratados como **internos**;
- não devem ser considerados API pública estável;
- mudanças estruturais devem ser refletidas neste documento e em `docs/architecture.md`.
