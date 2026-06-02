<!-- Caminho relativo: docs/upstream-capabilities-spike.md -->

# Spike: capacidades reais do upstream `@mlightcad`

> Investigação técnica (2026-06-02) para validar o inventário real de comandos, camadas
> e seleção antes de implementar as Frentes 1–3 do
> [roadmap](./cad-panels-commands-simulation-roadmap.md). Decisão derivada:
> [ADR 0001](./adr/0001-catalogo-dinamico-e-paineis-sobre-data-model.md).

## Escopo e método

Inspeção das declarações de tipo (`.d.ts`) e do uso real em código de:

- `@mlightcad/cad-simple-viewer@1.5.0`
- `@mlightcad/data-model`

O adaptador atual (`src/lib/viewer/neocad-viewer.ts`) usa `AcApDocManager`,
`docManager.curView`, `docManager.events.documentActivated` e
`docManager.sendStringToExecute(command)`. O objetivo foi descobrir o que mais é
acessível a partir do `docManager`/documento ativo.

## 1. Inventário de comandos

### Resolução e enumeração

- `docManager.commandManager: AcEdCommandStack`.
- `sendStringToExecute(cmdStr)` resolve via `lookupGlobalCmd(cmdName)` (nome global) /
  nome local, com suporte a aliases (case-insensitive).
- **Enumeração em runtime confirmada:**
  - `commandStack.iterator(): AcEdCommandIterator` — percorre todos os comandos de todos
    os grupos.
  - `commandStack.searchCommandsByPrefix(prefix, mode?): AcEdCommandIteratorItem[]` —
    cada item tem `{ command: AcEdCommand, groupName: string }`.
  - `AcEdCommand.globalName` / `localName` expõem os nomes.

> Implicação: o catálogo de comandos da UI pode ser **derivado dinamicamente** do stack,
> sem lista hardcoded — fonte de verdade sempre fiel ao que o viewer aceita.

### Comandos registrados (por grupo)

| Grupo | Comandos (globalName) |
|-------|------------------------|
| Navegação/sistema | `ZOOM`, `PAN`, `SELECT`, `OPEN`, `QNEW`, `REGEN`, `SWITCHBG`, `SYSVAR`, `LOG` |
| Desenho | `LINE`, `CIRCLE`, `ARC`, `ELLIPSE`, `RECT`, `PLINE`, `POLYGON`, `SPLINE`, `POINT`, `RAY`, `XLINE`, `MLINE`, `MTEXT`, `DIMLINEAR`, `HATCH` |
| Edição | `ERASE`, `MOVE`, `COPY`, `ROTATE` |
| Camadas | `LAYER`, `LAYCUR`, `LAYON`, `LAYOFF`, `LAYFRZ`, `LAYTHW`, `LAYLCK`, `LAYULK`, `LAYISO`, `LAYUNISO`, `LAYP`, `LAYDEL`, `LAYCLOSE` |

> Os `globalName` acima vêm das definições dos comandos; **confirmar a string exata em
> runtime** iterando o stack antes de fixar no catálogo de apresentação.

### Ausências relevantes

`UNDO`/`REDO`, `SCALE`, `MIRROR`, `ARRAY`, `OFFSET`, `TRIM`/`EXTEND`, `BLOCK`/`INSERT`,
`PROPERTIES`/`CHPROP`.

## 2. Tabela de camadas

### Caminho de acesso

```text
docManager.curDocument            // AcApDocument
  .database                       // AcDbDatabase
  .tables.layerTable              // AcDbLayerTable  (extends AcDbSymbolTable)
```

### Iteração e busca (confirmadas em `AcDbSymbolTable`)

- `newIterator(): AcDbObjectIterator<AcDbLayerTableRecord>` — iterável com `for...of`.
- `numEntries: number`, `getAt(name)`, `has(name)`, `getIdAt(id)`.

### `AcDbLayerTableRecord` — getters **e** setters

`name`, `color` (`AcCmColor`), `description`, `isOff`, `isFrozen`, `isLocked`,
`isHidden`, `isPlottable`, `isInUse`, `linetype`, `lineWeight`, `transparency`,
`standardFlags`. Read-only: `lineStyle`.

### Eventos

`database.events.layerModified` (`AcDbLayerModifiedEventArgs`), `layerAppended`.

> Leitura e escrita disponíveis; o painel inicia em modo leitura (ADR 0001 / roadmap),
> escrita habilitada incrementalmente.

## 3. Seleção e propriedades de entidade

### Seleção

```text
docManager.curView                // AcTrView2d (extends AcEdBaseView)
  .selectionSet                   // AcEdSelectionSet
```

- `ids: string[]`, `count: number`, `has(id)`, `add(id|id[])`, `delete(id|id[])`,
  `clear()`.
- Eventos: `selectionSet.events.selectionAdded` / `selectionRemoved`
  (`AcEdSelectionEventArgs { ids }`).
- View também expõe `events.hover` / `unhover`.

### Resolução id → entidade (padrão usado pelo próprio upstream)

```ts
const entity = docManager.curDocument.database.tables.blockTable.getEntityById(id);
// AcDbEntity | undefined
```

### `AcDbEntity` — propriedades genéricas

Getters: `type`, `dxfTypeName`, `resolvedColor`, `rgbColor`, `objectId`, `handle`,
`properties`. Get/set: `layer`, `color`, `lineType`, `lineWeight`, `linetypeScale`,
`visibility`, `transparency`.

### Modelo pronto para UI — `AcDbEntityProperties`

```ts
interface AcDbEntityProperties { type: string; groups: AcDbEntityPropertyGroup[]; }
interface AcDbEntityPropertyGroup { groupName: string; properties: AcDbEntityRuntimeProperty[]; }
// cada propriedade: name, type ('string'|'int'|'float'|'enum'|'color'|'layer'|...),
// editable?, accessor { get(); set?() }
```

> O painel de Propriedades pode renderizar `entity.properties.groups` diretamente,
> usando `accessor.get()`/`set()` e `type` para escolher o controle.

## Conclusões para implementação

1. **Frente 2 (catálogo + `Ajuda`)** — derivar do `AcEdCommandStack`; o catálogo
   estático guarda só metadados de apresentação PT-BR (rótulo, categoria, ícone).
2. **Frente 1 (painéis)** — `cad-layers.ts` lê `layerTable.newIterator()`;
   `cad-selection.ts` escuta `selectionSet.events` e resolve via `getEntityById`,
   expondo `entity.properties` ao painel.
3. **Frente 3 (comandos básicos)** — disparar por `sendStringToExecute(globalName)`;
   expor na UI apenas comandos presentes no stack.
4. Toda essa superfície fica **isolada atrás do adaptador `NeoCadViewer` e dos
   serviços** (ADR 0001): componentes Svelte não importam `@mlightcad/*`.
