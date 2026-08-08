<!-- Caminho relativo: docs/api.md -->

# API e contratos internos

## Objetivo

Registra os contratos internos do NeoCAD. Todos são **internos** e sujeitos a
evolução — não há API pública remota, e não haverá antes do MVP.

A exceção prevista é o kernel: o [ADR 0003](./adr/0003-kernel-cad-proprio.md) o
trata como biblioteca reutilizável, e a sua API pública passa a exigir
disciplina de versionamento quando for extraída para repositório próprio. Até lá,
também é interna.

## Fronteira de integração

Vale o [ADR 0001](./adr/0001-catalogo-dinamico-e-paineis-sobre-data-model.md):
componentes Svelte e rotas consomem **apenas** os contratos de
`src/lib/types/cad.ts`. Nem `@mlightcad/*` nem `$lib/kernel` aparecem fora do
adaptador e dos serviços.

## Contratos do documento

Tipos em `src/lib/types/cad.ts`.

| Contrato                    | Papel                                              |
| --------------------------- | -------------------------------------------------- |
| `CadLayerId`, `CadEntityId` | Identificadores opacos, com `brand` distinto       |
| `CadPoint`, `CadBounds`     | Ponto e caixa envolvente                           |
| `CadColor`                  | Índice ACI ou cor verdadeira                       |
| `CadLayer`                  | Camada com nome, cor e estados                     |
| `CadGeometry`               | Linha, círculo, arco, polilinha ou texto           |
| `CadEntity`                 | Entidade com camada, geometria e extensão          |
| `CadHistoryState`           | Estado da pilha, para o menu `Editar`              |
| `CadDocumentSnapshot`       | Retrato extraído do upstream, pronto para o kernel |
| `CadLoadReport`             | Contagens do que o kernel recebeu                  |

Os identificadores são `string` com `brand` distinto de propósito: o kernel
separa camada de entidade em tipos, e perder essa distinção ao cruzar a ponte
faria o compilador aceitar a troca entre elas.

## Serviços

### Fronteira com o kernel

`src/lib/services/cad-document.ts` — **única** porta de acesso ao kernel.

Classe `CadDocument`:

- `create()` — carrega o WebAssembly e cria um documento vazio;
- `load(snapshot)` — substitui o documento e zera o histórico;
- `listLayers()`, `listEntities()`, `countEntities()`, `getBounds()`;
- `getHistory()`;
- `createLayer(name)`, `drawLine(layer, start, end)`, `eraseEntity(id)`,
  `setLayerOff(layer, off)`;
- `undo()`, `redo()`.

Conversores puros, exportados para teste sem navegador:

- `toCadLayer`, `toCadEntity`, `toCadGeometry`, `toCadBounds`, `toCadColor`,
  `toCadHistoryState` — do kernel para os contratos NeoCAD;
- `buildDocumentSnapshot`, `toCadGeometryFromUpstream`, `toCadLayerSnapshot`,
  `toCadColorFromUpstream` — do upstream para o retrato do documento.

Uma forma inesperada vinda do kernel lança `CadKernelContractError` com o caminho
do campo. É defeito do kernel, não entrada não confiável, e falhar alto evita que
`undefined` se espalhe pela interface.

### Adaptador do viewer

`src/lib/viewer/neocad-viewer.ts`.

- `mount(container)`, `destroy()`;
- `openDocument(payload, mode)`;
- `zoomToFit()`, `toggleBackground()`, `executeCommand(command)`;
- `listCommandDescriptors()` — inventário de runtime do command stack upstream;
- `extractDocumentSnapshot()` — retrato do documento aberto, para o kernel.

Aponta para os workers de DXF, DWG e MTEXT em `static/workers/`, derivados de
`node_modules` no build.

### Arquivos CAD

`src/lib/services/cad-file.ts` — `selectCadDocument()`,
`readCadDocumentFromPath(path)`, `createCadDocumentPayloadFromFile(file)`,
`extractCadFileName(path)`, `isSupportedCadFile(fileName)`,
`getCadRuntimeLabel()`.

### Recentes

`src/lib/services/recent-documents.ts` — `listRecentDocuments()`,
`registerRecentDocument(document)`, `clearRecentDocuments()`. Persiste em
`AppConfig/state/recent-documents.json` no Tauri, com fallback em `localStorage`.

### Catálogo de comandos

`src/lib/services/cad-commands.ts` e `src/lib/config/cad-command-catalog.ts`.
O catálogo é derivado em runtime do command stack do upstream (ADR 0001), com
metadados de apresentação em PT-BR sobrepostos.

## API do kernel

Superfície exposta por `neocad-wasm`, consumida somente por `cad-document.ts`.

Classe `CadSession` — documento e histórico juntos, para não haver como desfazer
contra o documento errado:

- `layers()`, `entities()`, `entityCount()`, `boundingBox()`, `history()`;
- `load(document)`;
- `createLayer(name)`, `addLine(...)`, `removeEntity(id)`, `setLayerOff(...)`;
- `undo()`, `redo()`.

Identificadores atravessam a ponte como **texto decimal**: um `u64` viraria
`BigInt` do lado JavaScript, o que complica comparação e serialização sem ganho —
o identificador é opaco de qualquer modo.

### Invariantes do kernel

- alterar entidades ou propriedades de camada só é possível por
  `Document::edit()`, que registra a inversa de cada operação;
- desfazer restaura o **mesmo** identificador e a **mesma** posição na ordem de
  desenho, e não algo equivalente;
- entidade só entra no documento se a camada existir;
- remover camada com entidades é recusado; remover bloco leva as entidades dele.

## Eventos consumidos do upstream

`eventBus` do `cad-simple-viewer`: `open-file`, `open-file-progress`, `message`,
`failed-to-open-file`, `font-not-found`.

## Plugins Tauri

| Plugin   | Uso                               | Permissão           |
| -------- | --------------------------------- | ------------------- |
| `dialog` | Seletor nativo de arquivos CAD    | `dialog:allow-open` |
| `fs`     | Leitura do arquivo e estado local | `fs:read-files`     |

## Contratos previstos

### Painéis de camadas e propriedades (Frente 1)

Consumirão `CadLayer` e `CadEntity` do kernel. Áreas prováveis:
`src/lib/services/cad-layers.ts`, `cad-selection.ts` e componentes em
`src/lib/components/workspace/`.

O modo escrita depende de as tabelas de símbolos ganharem operações reversíveis —
criar, renomear e remover camada ainda não passam pelo command stack.

### Persistência de arquivo (K2)

`neocad-io` ganhará leitura e escrita DXF nativas, e com elas `Salvar`,
`Salvar como` e `Exportar`.

### Interface headless (ADR 0004)

`neocad-cli` como núcleo funcional e `neocad-mcp` como fachada MCP, ambos sobre
o kernel, depois de K2.

## Estabilidade

- os contratos aqui descritos são **internos** e não constituem API pública;
- mudanças estruturais se refletem neste documento e em
  [`architecture.md`](./architecture.md);
- quebra de contrato do CLI ou do esquema MCP, quando existirem, é registrada no
  `CHANGELOG.md` (ADR 0004).
