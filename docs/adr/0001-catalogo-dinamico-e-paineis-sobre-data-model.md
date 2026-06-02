<!-- Caminho relativo: docs/adr/0001-catalogo-dinamico-e-paineis-sobre-data-model.md -->

# ADR 0001: Catálogo de comandos derivado do upstream e painéis de leitura sobre o `data-model`

- **Status:** Accepted <!-- Proposed | Accepted | Deprecated | Superseded by ADR-MMMM -->
- **Data:** 2026-06-02
- **Decisores:** Iago Leal
- **Tags:** upstream, comandos, camadas, seleção, fronteira-de-stack

## Contexto

O roadmap `docs/cad-panels-commands-simulation-roadmap.md` definiu três frentes
funcionais (painéis de camadas/propriedades, catálogo de comandos no menu `Ajuda`,
comandos CAD básicos) sob a premissa cautelosa de **"não prometer comandos que o
upstream não executa de forma confiável"** e de que a API de camadas/seleção do
`@mlightcad/cad-simple-viewer` ainda precisava ser validada antes de qualquer UI.

Um spike técnico (2026-06-02) inspecionou as declarações de tipo (`.d.ts`) e o uso real
no código de `@mlightcad/cad-simple-viewer@1.5.0` e `@mlightcad/data-model`, confirmando:

- **Comandos:** o `docManager.commandManager` (`AcEdCommandStack`) é **iterável em
  runtime** via `iterator()` e `searchCommandsByPrefix(prefix, mode)`, e cada comando
  expõe `globalName`/`localName`/`groupName`. Há ~31 comandos registrados, incluindo
  desenho (`LINE, CIRCLE, ARC, ELLIPSE, RECT, PLINE, POLYGON, SPLINE, POINT, MTEXT,
HATCH`…), edição (`ERASE, MOVE, COPY, ROTATE`) e camadas (`LAYER, LAYON/OFF,
LAYFRZ/THW, LAYLCK/ULK, LAYISO`…). **Ausentes:** `UNDO/REDO, SCALE, MIRROR, ARRAY,
OFFSET, TRIM/EXTEND, BLOCK/INSERT`.
- **Camadas:** acessíveis em `docManager.curDocument.database.tables.layerTable`
  (`AcDbLayerTable`), iteráveis por `newIterator()`/`getAt(name)`; os registros expõem
  getters e setters (`name, color, isOff, isFrozen, isLocked, linetype, lineWeight,
transparency`). Eventos `database.events.layerModified`/`layerAppended`.
- **Seleção e propriedades:** `docManager.curView.selectionSet` (`ids`, `count`,
  eventos `selectionAdded`/`selectionRemoved`); resolução id→entidade por
  `database.tables.blockTable.getEntityById(id)`; `entity.properties` retorna um modelo
  `AcDbEntityProperties` (`{ type, groups[] }`) já estruturado para UI.

Detalhamento completo em `docs/upstream-capabilities-spike.md`.

A premissa de incerteza do roadmap deixou de valer: a realidade do upstream é
**descobrível programaticamente**. Uma decisão precisa ser tomada agora para fixar
**como** a UI do NeoCAD consome essas capacidades, antes de escrever os serviços e
componentes das Frentes 1 e 2 — evitando divergência entre o que a UI promete e o que o
viewer realmente faz.

## Decisão

Fica acordado que:

1. O **catálogo de comandos** exibido pela UI do NeoCAD (menu `Ajuda` e futuras
   toolbars) será **derivado em runtime** do `AcEdCommandStack` do upstream, e não
   mantido como lista hardcoded. Um catálogo estático de metadados de apresentação
   (rótulo amigável PT-BR, categoria, ícone) **pode** existir, mas a fonte de verdade
   sobre _quais comandos existem_ é sempre o command stack consultado em tempo de
   execução.

2. Os **painéis de camadas e propriedades** (Frente 1) serão construídos diretamente
   sobre a API pública do `@mlightcad/data-model` (`AcDbLayerTable`,
   `AcDbLayerTableRecord`, `AcEdSelectionSet`, `AcDbEntity.properties`), **mediados pelo
   adaptador `NeoCadViewer`** (`src/lib/viewer/neocad-viewer.ts`). Os componentes Svelte
   **não** importam tipos do upstream diretamente; consomem contratos NeoCAD
   (`src/lib/types/cad.ts`, serviços em `src/lib/services/`).

3. A **fronteira de integração** com o upstream fica concentrada no adaptador e nos
   serviços (`cad-layers.ts`, `cad-selection.ts`, `cad-commands.ts`). Toda a superfície
   do `@mlightcad/*` é tratada como **API interna não-estável**, isolada atrás dessa
   fronteira para que uma futura troca/fork de upstream não vaze para a UI.

## Consequências

- **Impacto positivo:** o menu `Ajuda` nunca diverge do que o viewer aceita; as Frentes
  1, 2 e 3 ficam desbloqueadas com API confirmada de ponta a ponta; o painel de
  propriedades reaproveita o modelo `AcDbEntityProperties` pronto, reduzindo trabalho.
- **Impacto negativo:** acoplamento à forma do `AcEdCommandStack`/`data-model`, que são
  internos do upstream e podem mudar entre versões; a iteração de comandos pode expor
  comandos sem rótulo amigável (exigem mapeamento de apresentação).
- **Trade-offs aceitos:** depender de APIs upstream não documentadas como estáveis, em
  troca de fidelidade automática entre UI e capacidades reais. O isolamento atrás do
  adaptador limita o raio de impacto de mudanças do upstream.

## Diretriz de Conformidade de Código

- **Proibido:** importar tipos/símbolos de `@mlightcad/cad-simple-viewer` ou
  `@mlightcad/data-model` diretamente em componentes Svelte (`src/lib/components/**`) ou
  rotas (`src/routes/**`). Esses imports só são permitidos em
  `src/lib/viewer/neocad-viewer.ts` e nos serviços de `src/lib/services/`.
- **Proibido:** manter no código uma lista hardcoded de comandos como fonte de verdade
  sobre disponibilidade; manter apenas metadados de apresentação opcionais.
- **Proibido:** expor na UI comandos do grupo ausente (`UNDO/REDO, SCALE, MIRROR,
ARRAY, OFFSET, TRIM/EXTEND, BLOCK/INSERT`) como se fossem implementados, enquanto não
  existirem no command stack.
- **Obrigatório:** todo acesso a camadas, seleção e propriedades de entidade passa pelo
  adaptador `NeoCadViewer` e/ou pelos serviços de `src/lib/services/`, expondo contratos
  NeoCAD em `src/lib/types/cad.ts`.

> Qualquer tentativa de desvio desta regra viola as diretrizes de conformidade
> arquitetural do projeto e deve ser reportada ao operador humano antes de prosseguir.
