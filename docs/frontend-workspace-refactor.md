<!-- Caminho relativo: docs/frontend-workspace-refactor.md -->

# Plano de refatoração do workspace frontend

## Objetivo

Este documento descreve o plano de implementação que guiou a refatoração do frontend do NeoCAD, reduzindo a concentração de responsabilidades em `src/routes/+page.svelte` e centralizando os estilos CSS em uma estrutura mais previsível.

A proposta mantém `+page.svelte` como **orquestrador da tela principal**, enquanto extrai telas, painéis e blocos de UI para componentes Svelte dedicados.

## Status atual

A refatoração descrita aqui foi **implementada na Fase 2** e este documento passa a servir como referência da estrutura adotada.

Resultado atual resumido:

- `src/routes/+page.svelte` atua como controlador do workspace desktop;
- o frontend usa componentes específicos em `src/lib/components/workspace/`;
- os estilos compartilhados foram centralizados em `src/lib/styles/`;
- o topo evoluiu para um menu superior mais próximo de aplicações desktop;
- o canvas passou a ter maior prioridade visual, mantendo a barra de comandos do viewer upstream.

## Problema atual

No estado atual, `src/routes/+page.svelte` concentra ao mesmo tempo:

- estado do workspace;
- integração com `NeoCadViewer`;
- markup de múltiplas telas;
- ações globais da interface;
- dock de mensagens;
- topbar;
- estilos globais e específicos do workspace.

Esse acoplamento dificulta:

- manutenção incremental;
- leitura do fluxo principal;
- reaproveitamento de blocos visuais;
- evolução da UX desktop com mais área útil para o canvas CAD.

## Objetivos da refatoração

1. transformar `+page.svelte` no ponto de controle do workspace;
2. extrair telas e painéis para componentes dedicados;
3. introduzir um menu superior mais próximo do padrão desktop;
4. reduzir redundâncias visuais no workspace do viewer;
5. centralizar estilos compartilhados em `src/lib/styles/`;
6. preservar a integração existente com `NeoCadViewer`, Tauri e serviços atuais.

## Fora de escopo inicial

Nesta etapa, o plano **não** pretende:

- alterar a integração de baixo nível com `@mlightcad/cad-simple-viewer`;
- introduzir gerenciamento global de estado com stores além do necessário;
- reescrever serviços de arquivos CAD ou persistência de recentes;
- implementar novos comandos CAD complexos;
- redesenhar toda a identidade visual do app.

## Diretrizes arquiteturais

- `src/routes/+page.svelte` continua responsável por estado, ciclo de vida e integração do viewer;
- componentes recebem **props simples** e disparam ações por **callbacks** explícitos;
- detalhes institucionais e descritivos devem sair do viewer principal e migrar para telas auxiliares;
- o CSS compartilhado deve ser movido para arquivos centralizados, evitando repetição de tokens, botões, cards, chips e grids;
- o refactor deve ocorrer em etapas pequenas, mantendo o app funcional ao fim de cada fase.

## Estrutura-alvo de diretórios

```text
src/
├── lib/
│   ├── components/
│   │   └── workspace/
│   │       ├── AboutScreen.svelte
│   │       ├── AppTopMenu.svelte
│   │       ├── HomeScreen.svelte
│   │       ├── MessagesDock.svelte
│   │       ├── RecentDocumentsPanel.svelte
│   │       ├── ViewerDropzone.svelte
│   │       ├── ViewerScreen.svelte
│   │       ├── ViewerToolbar.svelte
│   │       └── types.ts
│   ├── config/
│   ├── services/
│   ├── styles/
│   │       ├── base.css
│   │       ├── components.css
│   │       ├── index.css
│   │       ├── layout.css
│   │       ├── tokens.css
│   │       ├── utilities.css
│   │       └── workspace.css
│   ├── types/
│   └── viewer/
└── routes/
    ├── +layout.svelte
    ├── +layout.ts
    └── +page.svelte
```

## Plano por arquivo

### `src/routes/+layout.svelte`

**Mudança planejada:** importar `src/lib/styles/index.css` uma única vez no layout global.

**Responsabilidade final:**

- aplicar favicon, título e metadados globais;
- carregar a folha de estilos centralizada do app;
- renderizar o conteúdo das rotas.

**Observação:** esse arquivo passa a ser o ponto mais adequado para registrar CSS global compartilhado do frontend.

### `src/routes/+page.svelte`

**Mudança planejada:** reduzir markup e estilos locais, mantendo apenas a orquestração do workspace.

**Responsabilidade final:**

- manter estado principal do app;
- inicializar e conectar `NeoCadViewer` ao container do canvas;
- coordenar abertura de arquivos, recentes, drag-and-drop e mensagens;
- selecionar qual tela do workspace será renderizada;
- passar props e callbacks para componentes filhos.

**Estado esperado neste arquivo:**

- `activeWorkspace`;
- `currentDocument`;
- `recentDocuments`;
- `progress`;
- `notifications`;
- `backgroundTheme`;
- `isOpening`;
- `isViewerReady`;
- `isDragActive`;
- `isMessagesVisible`;
- `unreadMessages`.

**Ações que devem continuar centralizadas:**

- `openCadDrawing()`;
- `openRecentDrawing()`;
- `handleFileDrop()`;
- `fitDrawingToView()`;
- `toggleViewerBackground()`;
- `refreshRecentDocuments()`;
- `clearRecentDocumentsList()`;
- inicialização do `NeoCadViewer` em `onMount`.

## Componentes do workspace

### `src/lib/components/workspace/types.ts`

**Criar.**

**Responsabilidade:** centralizar tipos leves do workspace que não pertencem ao domínio CAD.

**Conteúdo inicial sugerido:**

```ts
export type WorkspaceView = 'home' | 'viewer' | 'about';
```

### `src/lib/components/workspace/AppTopMenu.svelte`

**Criar.**

**Responsabilidade:** menu superior no estilo desktop, com foco em navegação e ações globais.

**Props sugeridas:**

- `activeWorkspace`;
- `currentDocumentTitle?: string | null`;
- `hasVisitedViewerWorkspace`;
- `unreadMessages`;
- `isViewerReady`;
- `isOpening`;
- `recentDocuments`.

**Callbacks sugeridos:**

- `onGoHome()`;
- `onGoViewer()`;
- `onGoAbout()`;
- `onOpenDrawing()`;
- `onOpenRecent(item)`;
- `onClearRecents()`;
- `onFitView()`;
- `onToggleBackground()`;
- `onToggleMessages()`.

**Menus sugeridos:**

- `Arquivo` → abrir desenho, abrir recente, limpar recentes;
- `Exibir` → ajustar vista, alternar fundo, mostrar/ocultar mensagens;
- `Janela` → início, canvas CAD, sobre;
- `Ajuda` → sobre o NeoCAD.

### `src/lib/components/workspace/HomeScreen.svelte`

**Criar.**

**Responsabilidade:** tela inicial do workspace, mais enxuta e orientada à entrada no canvas.

**Props sugeridas:**

- `runtimeLabel`;
- `isViewerReady`;
- `isMessagesVisible`;
- `isOpening`;
- `recentDocuments`;
- `isTauriRuntime`.

**Callbacks sugeridos:**

- `onOpenDrawing()`;
- `onEnterViewer()`;
- `onOpenRecent(item)`;
- `onClearRecents()`.

**Observação de UX:** reduzir texto institucional e priorizar CTA para abrir desenho.

### `src/lib/components/workspace/RecentDocumentsPanel.svelte`

**Criar.**

**Responsabilidade:** listar recentes, exibir estado vazio e acionar reabertura/limpeza.

**Props sugeridas:**

- `recentDocuments`;
- `isTauriRuntime`.

**Callbacks sugeridos:**

- `onOpenRecent(item)`;
- `onClearRecents()`.

**Observação:** esse componente deve ser reutilizável tanto na Home quanto no menu `Arquivo > Abrir recente`.

### `src/lib/components/workspace/ViewerScreen.svelte`

**Criar.**

**Responsabilidade:** compor a área principal do canvas, toolbar e estados visuais do viewer.

**Props sugeridas:**

- `currentDocument`;
- `backgroundTheme`;
- `progress`;
- `isViewerReady`;
- `isOpening`;
- `isDragActive`.

**Callbacks sugeridos:**

- `onOpenDrawing()`;
- `onFitView()`;
- `onToggleBackground()`;
- `onDragEnter(event)`;
- `onDragOver(event)`;
- `onDragLeave(event)`;
- `onDrop(event)`;
- `onViewerHostReady(element)`.

**Observação de implementação:** o container do viewer deve continuar sendo fornecido para o `NeoCadViewer` a partir de `+page.svelte`.

### `src/lib/components/workspace/ViewerToolbar.svelte`

**Criar.**

**Responsabilidade:** conter os botões rápidos mais importantes do canvas.

**Props sugeridas:**

- `isViewerReady`;
- `hasDocument`;
- `backgroundTheme`;
- `isOpening`.

**Callbacks sugeridos:**

- `onOpenDrawing()`;
- `onFitView()`;
- `onToggleBackground()`.

**Observação de UX:** essa toolbar deve ocupar menos altura que o header atual e ficar pronta para comandos CAD futuros.

### `src/lib/components/workspace/ViewerDropzone.svelte`

**Criar.**

**Responsabilidade:** encapsular o canvas, overlay sem documento e feedback visual de drag-and-drop.

**Props sugeridas:**

- `currentDocument`;
- `isDragActive`;
- `backgroundTheme`.

**Callbacks sugeridos:**

- `onDragEnter(event)`;
- `onDragOver(event)`;
- `onDragLeave(event)`;
- `onDrop(event)`;
- `onHostReady(element)`.

### `src/lib/components/workspace/AboutScreen.svelte`

**Criar.**

**Responsabilidade:** concentrar informações institucionais e técnicas que hoje ocupam espaço útil do viewer.

**Props sugeridas:**

- `appName`;
- `status`;
- `license`;
- `runtimeLabel`;
- `primaryStack`;
- `nextMilestones`;
- `supportedTargets`.

### `src/lib/components/workspace/MessagesDock.svelte`

**Criar.**

**Responsabilidade:** encapsular o dock recolhível de mensagens e o botão flutuante de acesso rápido.

**Props sugeridas:**

- `notifications`;
- `isVisible`;
- `unreadMessages`.

**Callbacks sugeridos:**

- `onOpen()`;
- `onClose()`.

## Centralização de estilos CSS

### `src/lib/styles/index.css`

**Criar.**

**Responsabilidade:** agregar as demais folhas de estilo via `@import`.

**Conteúdo sugerido:**

```css
@import './tokens.css';
@import './base.css';
@import './utilities.css';
@import './layout.css';
@import './components.css';
@import './workspace.css';
```

### `src/lib/styles/tokens.css`

**Criar.**

**Responsabilidade:** definir tokens globais de tema, spacing, radius, sombras e camadas.

**Exemplos de conteúdo:**

- cores de fundo, superfície e borda;
- cores de texto principal e secundário;
- espaçamentos padronizados;
- raios de borda;
- sombras para painéis;
- alturas base de topbar e dock.

### `src/lib/styles/base.css`

**Criar.**

**Responsabilidade:** reset leve e estilos base globais.

**Mover para cá:**

- estilos globais de `html` e `body`;
- `box-sizing` global;
- normalização de `button` e `input`;
- tipografia base.

### `src/lib/styles/utilities.css`

**Criar.**

**Responsabilidade:** classes utilitárias pequenas e reutilizáveis.

**Exemplos sugeridos:**

- `sr-only`;
- helpers de stack/cluster;
- `text-muted`;
- `scroll-y`.

### `src/lib/styles/layout.css`

**Criar.**

**Responsabilidade:** layout macro do app e regras de grid/responsividade.

**Mover para cá:**

- `.app-shell`;
- `.workspace-stage`;
- `.workspace-screen`;
- grids principais;
- breakpoints globais.

### `src/lib/styles/components.css`

**Criar.**

**Responsabilidade:** classes compartilhadas de componentes visuais.

**Mover para cá:**

- `.card-panel`;
- `.status-chip`;
- `.badge`;
- `.primary-button`;
- `.secondary-button`;
- `.inline-action`;
- `.label`;
- `.eyebrow`;
- listas padronizadas;
- pills e chips.

### `src/lib/styles/workspace.css`

**Criar.**

**Responsabilidade:** regras específicas do workspace NeoCAD.

**Mover para cá:**

- menu superior;
- toolbar do viewer;
- strip de metadados restantes;
- dock de mensagens;
- overlays do canvas;
- estados de drag-and-drop.

## Estratégia de migração

As etapas abaixo registram a sequência adotada durante a implementação. Elas continuam úteis como referência para futuras evoluções do workspace.

### Etapa 1 — preparar a base de estilos

1. criar `src/lib/styles/`;
2. mover estilos globais do `+page.svelte` para arquivos CSS dedicados;
3. importar `index.css` em `src/routes/+layout.svelte`;
4. manter `+page.svelte` funcional com markup original temporariamente.

### Etapa 2 — extrair componentes estáveis

1. criar `MessagesDock.svelte`;
2. criar `RecentDocumentsPanel.svelte`;
3. criar `AboutScreen.svelte`;
4. criar `HomeScreen.svelte`.

### Etapa 3 — extrair a área do viewer com cuidado

1. criar `ViewerToolbar.svelte`;
2. criar `ViewerDropzone.svelte`;
3. criar `ViewerScreen.svelte`;
4. preservar o vínculo entre o host DOM do canvas e `NeoCadViewer`.

### Etapa 4 — substituir a topbar por menu desktop-like

1. criar `AppTopMenu.svelte`;
2. migrar navegação e ações globais para submenus;
3. remover blocos duplicados de informação no topo e no viewer.

### Etapa 5 — limpeza final

1. reduzir o `<style>` residual de `+page.svelte` ao mínimo ou eliminá-lo;
2. revisar nomes de classes reutilizadas;
3. remover markup redundante;
4. atualizar testes E2E se a navegação ou rótulos mudarem.

## Critérios de aceite

A refatoração será considerada bem-sucedida quando:

- `src/routes/+page.svelte` estiver focado em estado, integração e composição;
- as telas principais estiverem separadas em componentes do diretório `src/lib/components/workspace/`;
- os estilos compartilhados estiverem centralizados em `src/lib/styles/`;
- o workspace do viewer ocupar mais área útil vertical;
- informações institucionais redundantes tiverem sido reduzidas;
- o menu superior tiver comportamento consistente com aplicações desktop;
- `pnpm check`, `pnpm lint` e `pnpm test` continuarem passando.

## Validação recomendada por etapa

Após cada grupo de mudanças, rodar:

```bash
pnpm check
pnpm lint
pnpm test
```

Quando a navegação ou hierarquia do DOM mudar de forma relevante, validar também:

```bash
pnpm test:e2e
```

## Riscos e mitigação

### Risco: quebrar a integração com o host do viewer

**Mitigação:** manter a inicialização do `NeoCadViewer` em `+page.svelte` até o fim da refatoração.

### Risco: espalhar CSS sem padrão

**Mitigação:** mover primeiro tokens, base e componentes visuais compartilhados antes de extrair o resto.

### Risco: introduzir excesso de abstração cedo demais

**Mitigação:** usar componentes pequenos, props simples e callbacks explícitos, sem stores novos neste primeiro ciclo.

## Resultado esperado

Ao final desse trabalho, o frontend do NeoCAD ficará mais próximo de uma arquitetura desktop sustentável:

- tela principal mais limpa;
- canvas CAD com mais espaço útil;
- menu superior mais adequado ao domínio do produto;
- componentes reaproveitáveis;
- CSS centralizado e mais fácil de evoluir.
