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
- persistir recentes em `AppConfig/state/recent-documents.json` no runtime Tauri;
- manter fallback em `localStorage` no navegador e para migração leve do estado web;
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

## Contratos planejados para a próxima etapa

### Catálogo de comandos CAD

Planeja-se introduzir um catálogo interno de comandos para alimentar:

- menu `Ajuda` com lista dos comandos implementados;
- futura toolbar de desenho/edição;
- possíveis atalhos de teclado.

Contrato sugerido:

- `CadCommandCatalogItem`
- `listImplementedCadCommands()`
- `listPlannedCadCommands()`
- `executeCadCommand(commandId)`

### Painéis de camadas e propriedades

Planeja-se investigar contratos internos para:

- leitura da tabela de camadas do documento ativo;
- estado de seleção de entidades;
- propriedades de documento e entidades.

Prováveis áreas de implementação:

- `src/lib/services/cad-layers.ts`
- `src/lib/services/cad-selection.ts`
- componentes em `src/lib/components/workspace/`

### Trilha de simulação numérica

Para FEM/CFD, a recomendação atual é planejar contratos internos separados do núcleo CAD, envolvendo:

- preparação de casos;
- execução externa de engine por Tauri/Rust;
- leitura de resultados e pós-processamento.

Essa trilha ainda deve ser tratada como pesquisa arquitetural e não como API estável.

## Comandos futuros ainda previstos

Ainda faz sentido manter no radar:

- preferências do usuário;
- exportações;
- integração de salvamento;
- catálogo de comandos CAD;
- painéis de propriedades e camadas;
- integração opcional de simulação numérica.

Mas esses contratos ainda não foram implementados como API estável do app.

## Estabilidade

Até a entrega do MVP:

- os contratos aqui descritos devem ser tratados como **internos**;
- não devem ser considerados API pública estável;
- mudanças estruturais devem ser refletidas neste documento e em `docs/architecture.md`.
