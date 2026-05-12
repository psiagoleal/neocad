<!-- Caminho relativo: docs/api.md -->

# API e contratos iniciais

## Objetivo

Este documento registra os contratos internos planejados para NeoCAD. Nesta fase, eles são **propostos**, não estáveis.

## Escopo

NeoCAD, no MVP, não terá uma API pública remota. O foco é definir contratos internos entre:

- UI SvelteKit;
- serviços de aplicação;
- adaptador do `cad-viewer`;
- comandos Tauri.

## Contratos internos propostos

### Serviço de documentos

Responsabilidades planejadas:

- abrir arquivo local;
- manter documento atual;
- controlar lista de recentes;
- sinalizar estado de carregamento e falha.

Interface conceitual:

- `openFromDialog()`
- `openFromPath(path)`
- `reloadCurrent()`
- `closeCurrent()`
- `listRecentFiles()`

### Serviço de viewer

Responsabilidades planejadas:

- inicializar o `cad-viewer` em um container controlado;
- carregar documento no viewer;
- expor operações de visualização;
- conectar comandos básicos de edição quando disponíveis.

Interface conceitual:

- `mount(container)`
- `loadDocument(source)`
- `fitToView()`
- `zoomIn()`
- `zoomOut()`
- `selectAll()`
- `deleteSelection()`

### Serviço de preferências

Responsabilidades planejadas:

- tema;
- idioma;
- comportamento da janela;
- opções de visualização e painel.

Interface conceitual:

- `getPreferences()`
- `savePreferences(partial)`
- `resetPreferences()`

## Comandos Tauri planejados

Os nomes finais poderão mudar, mas a separação por responsabilidade deve permanecer.

### Arquivos

- `open_file_dialog`
- `read_file_bytes`
- `save_file_dialog`
- `list_recent_files`
- `register_recent_file`

### Aplicação

- `get_app_info`
- `get_platform_info`
- `open_external_url`

### Configurações

- `load_preferences`
- `save_preferences`

## Eventos planejados

Eventos internos ou bridges entre camadas:

- `document:opened`
- `document:failed`
- `document:closed`
- `viewer:ready`
- `viewer:selection-changed`
- `preferences:updated`

## Compatibilidade com o upstream

O adaptador do NeoCAD deverá:

- esconder detalhes internos do `cad-viewer` da maior parte da aplicação;
- concentrar eventuais quebras de API quando o upstream mudar;
- facilitar testes por mocking.

## Estabilidade

Até a entrega do MVP:

- os contratos aqui descritos devem ser tratados como **internos**;
- não devem ser considerados API pública estável;
- mudanças estruturais devem ser refletidas neste documento e em `docs/architecture.md`.
