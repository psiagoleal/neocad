<!-- Caminho relativo: docs/tickets/k2-dxf-nativo.md -->

# K2 — Leitura e escrita DXF nativas: micro-tickets

Quebra da fase **K2** do [ADR 0003](../adr/0003-kernel-cad-proprio.md), com a
leitura acrescentada pelo [ADR 0004](../adr/0004-interface-para-agentes-de-ia.md).
Formato conforme a skill `micro-ticket-planner`.

## Resultado esperado de K2

`neocad-io` lê e escreve DXF sem depender do upstream. Com isso o ciclo **abrir →
editar → salvar** fecha na interface, e o núcleo headless do
[ADR 0004](../adr/0004-interface-para-agentes-de-ia.md) passa a ser possível — um
CLI não pode depender de um parser que só roda em navegador.

## Por que agora, e não só por independência

A validação contra desenhos reais mostrou que **o parser DXF do upstream falha em
arquivos cuja seção `BLOCKS` contém bloco com entidades** — cerca de 11% do
acervo medido, justamente a fatia dos desenhos acabados, com carimbo e
simbologia. K2 deixa de ser preparação para o futuro e passa a corrigir um
defeito observado. A fixture `e2e/fixtures/block-with-entities.dxf` já registra o
caso, com `test.fail()` que quebra quando isto funcionar.

## O que K2 **não** faz

Não amplia o conjunto de entidades que o modelo representa. Hoje são `Line`,
`Circle`, `Arc`, `Polyline` e `Text`; o restante continua **contado e reportado
como não modelado**, sem impedir a abertura.

A validação mediu o que falta, por peso em desenho real — `DIMENSION` (176 num só
arquivo), `SPLINE` (64), `HATCH`, `INSERT`, `MTEXT`, `LEADER`, `SOLID`, `ATTDEF`.
Essa lista é insumo para priorizar uma fase de ampliação do modelo, que é
trabalho distinto de I/O e merece tickets próprios.

## Requisito superveniente: o layout é o que gera o documento

Registrado em 2026-08-11, vindo do usuário por outra sessão do ecossistema e
**pendente de confirmação direta**: o NeoCAD precisa trabalhar com layouts como o
AutoCAD, inclusive lendo os do AutoCAD, "pois é exatamente isto que gera os
documentos". No domínio de LT o entregável não é o desenho no espaço-modelo — é a
prancha composta no espaço-papel, com carimbo, escalas e viewports.

A medição que originou isso está no handoff: dos quatro desenhos reais validados,
um é **inteiramente** montado no papel e os outros três têm carimbo e viewports
lá. Ler só o espaço-modelo desenha, mas não emite documento.

**O que isso muda em K2, e é barato agora:** no DXF as entidades de papel não
estão em lugar separado — vivem na mesma seção `ENTITIES`, marcadas pelo código
`67` (`1` = espaço-papel) e pelo `410` (nome da aba de layout). Ler o espaço de
cada entidade custa dois códigos de grupo; **descartá-lo agora obriga a reescrever
o leitor depois**. Os tickets MT-K2-04 e MT-K2-08 devem preservar essa informação,
mesmo que o modelo ainda não a use.

**O que continua fora de K2, e é caro:** os objetos `LAYOUT` da seção `OBJECTS`
(configuração de página, escala de plotagem), a entidade `VIEWPORT` (a janela para
o espaço-modelo, com escala, rotação e recorte) e a composição da prancha. Isso é
fase própria, não ticket avulso, e depende de decisão do usuário sobre prioridade.

## Restrições que valem para todos os tickets

- `neocad-io` é a **única** crate do kernel autorizada a receber dependência
  copyleft (ADR 0003). Se a implementação for própria, melhor ainda.
- A escrita é **determinística**: o mesmo documento produz bytes idênticos
  (ADR 0004). Sem isso não há diff legível entre versões.
- Arquivo malformado **não derruba a abertura**: o que não for compreendido é
  contado e reportado, como já vale para a extração do upstream.
- Nenhum arquivo de origem confidencial entra no repositório (`AGENTS.md` §0.1);
  as fixtures são sintéticas.

---

## Bloco A — Fundação do formato

### MT-K2-01: Ler o fluxo de pares código/valor

- **Objetivo:** leitor de baixo nível que transforma bytes DXF numa sequência de
  pares `(código, valor)`, com o valor já tipado conforme a faixa do código.
- **Arquivos no escopo:** `kernel/neocad-io/src/{lib.rs,dxf/mod.rs,dxf/pairs.rs}`.
- **Critério de aceite:** `cargo test -p neocad-io` cobre inteiro, real, texto e
  binário; `CRLF` e `LF`; comentário `999`; espaços à esquerda do código; e
  arquivo truncado, que deve virar erro nomeado e não pânico.
- **Fora de escopo:** semântica de seções; DXF binário.
- **Depende de:** K1.

### MT-K2-02: Percorrer as seções

- **Objetivo:** máquina de seções (`HEADER`, `TABLES`, `BLOCKS`, `ENTITIES`,
  `OBJECTS`, `EOF`), entregando ao chamador os pares de cada uma.
- **Arquivos no escopo:** `kernel/neocad-io/src/dxf/{mod.rs,sections.rs}`.
- **Critério de aceite:** `cargo test -p neocad-io` cobre seção desconhecida
  (ignorada, não fatal), `ENDSEC` ausente, e ordem incomum de seções.
- **Fora de escopo:** interpretar o conteúdo das seções.
- **Depende de:** MT-K2-01.

---

## Bloco B — Leitura

### MT-K2-03: Ler a tabela de camadas

- **Objetivo:** `TABLES` → `LayerTable`, com nome, cor, tipo de linha, espessura
  e estados.
- **Arquivos no escopo:** `kernel/neocad-io/src/dxf/tables.rs`.
- **Critério de aceite:** `cargo test -p neocad-io` cobre cor por índice ACI,
  cor verdadeira, e os extremos `0`/`256` chegando como `ByBlock`/`ByLayer` —
  o defeito que dado real revelou.
- **Fora de escopo:** tabelas de bloco, estilo e tipo de linha.
- **Depende de:** MT-K2-02.

### MT-K2-04: Ler as entidades que o modelo representa

- **Objetivo:** `ENTITIES` → `Line`, `Circle`, `Arc`, `Polyline` e `Text`,
  incluindo a polilinha de estilo antigo (`POLYLINE`/`VERTEX`/`SEQEND`).
- **Arquivos no escopo:** `kernel/neocad-io/src/dxf/entities.rs`.
- **Critério de aceite:** `cargo test -p neocad-io` lê as fixtures sintéticas de
  `e2e/fixtures/` e confere contagem, camada e geometria de cada tipo.
- **Fora de escopo:** entidades fora do modelo; abaulamento de polilinha.
- **Depende de:** MT-K2-03.

### MT-K2-05: Ler a seção `BLOCKS`

- **Objetivo:** definições de bloco viram `BlockTable`, com as entidades de cada
  bloco no bloco correspondente.
- **Arquivos no escopo:** `kernel/neocad-io/src/dxf/blocks.rs`.
- **Critério de aceite:** `block-with-entities.dxf` — a fixture que o parser do
  upstream **não** lê — é lida corretamente, com o bloco e a entidade dentro
  dele. É o ticket que justifica a fase.
- **Fora de escopo:** resolver `INSERT` em geometria (exige transformação de
  instância, que é outra fase).
- **Depende de:** MT-K2-04.

### MT-K2-06: Contar o que não é compreendido

- **Objetivo:** entidade de tipo desconhecido é contada por tipo e ignorada, sem
  interromper a leitura; o relatório acompanha o documento.
- **Arquivos no escopo:** `kernel/neocad-io/src/dxf/{mod.rs,report.rs}`.
- **Critério de aceite:** `cargo test -p neocad-io` demonstra que um arquivo com
  `HATCH`, `DIMENSION` e `SPLICE` inventado abre, entrega as entidades válidas e
  reporta os três tipos com contagem.
- **Fora de escopo:** modelar essas entidades.
- **Depende de:** MT-K2-05.

---

## Bloco C — Escrita

### MT-K2-07: Escrever cabeçalho e tabelas

- **Objetivo:** serializar `HEADER` mínimo e `TABLES` a partir do documento.
- **Arquivos no escopo:** `kernel/neocad-io/src/dxf/writer/{mod.rs,tables.rs}`.
- **Critério de aceite:** a saída é **byte a byte idêntica** entre duas execuções
  sobre o mesmo documento.
- **Fora de escopo:** entidades.
- **Depende de:** MT-K2-03.

### MT-K2-08: Escrever entidades e blocos

- **Objetivo:** serializar `BLOCKS` e `ENTITIES`.
- **Arquivos no escopo:** `kernel/neocad-io/src/dxf/writer/entities.rs`.
- **Critério de aceite:** `cargo test -p neocad-io` confere a saída de cada tipo
  contra o esperado, e a determinismo entre execuções.
- **Fora de escopo:** preservar entidades que a leitura não compreendeu.
- **Depende de:** MT-K2-07, MT-K2-05.

### MT-K2-09: Fechar a ida e volta

- **Objetivo:** ler, escrever, reler e comparar.
- **Arquivos no escopo:** `kernel/neocad-io/tests/round_trip.rs`.
- **Critério de aceite:** para cada fixture sintética, o documento relido é igual
  ao original pela `PartialEq` semântica de `Document`. **A perda esperada é
  declarada no teste** — entidades não modeladas não sobrevivem à volta, e isso
  precisa estar escrito, não descoberto depois.
- **Fora de escopo:** preservação de conteúdo desconhecido.
- **Depende de:** MT-K2-08, MT-K2-06.

---

## Bloco D — Chegada ao produto

### MT-K2-10: Expor leitura e escrita na fachada

- **Objetivo:** `CadSession` ganha abrir DXF a partir de bytes e serializar o
  documento para DXF.
- **Arquivos no escopo:** `kernel/neocad-wasm/src/lib.rs`,
  `kernel/neocad-wasm/Cargo.toml`.
- **Critério de aceite:** `cargo test -p neocad-wasm` cobre os dois caminhos no
  host; `wasm-pack build` continua funcionando e o tamanho do `.wasm` é
  registrado — ele já dobrou uma vez sem ninguém notar.
- **Fora de escopo:** UI.
- **Depende de:** MT-K2-09.

### MT-K2-11: Ligar `Salvar` e `Salvar como` à interface

- **Objetivo:** o menu `Arquivo` ganha as duas ações, gravando pelo Tauri e por
  download no navegador.
- **Arquivos no escopo:** `src/lib/services/{cad-document.ts,cad-file.ts}`,
  `src/lib/components/workspace/AppTopMenu.svelte`, `src/routes/+page.svelte`,
  `src-tauri/capabilities/default.json`.
- **Critério de aceite:** `pnpm check` e `pnpm lint` verdes; a capability de
  escrita do Tauri é **restrita ao arquivo escolhido**, e não ampla.
- **Fora de escopo:** exportar para outros formatos.
- **Depende de:** MT-K2-10.

### MT-K2-12: Trocar a leitura do upstream pela nativa e virar a fixture

- **Objetivo:** a abertura de DXF passa a usar o kernel; o upstream segue
  desenhando.
- **Arquivos no escopo:** `src/lib/viewer/neocad-viewer.ts`,
  `src/lib/services/cad-document.ts`, `e2e/dxf-constructs.e2e.ts`.
- **Critério de aceite:** `pnpm test:e2e` passa **com o `test.fail()` removido**
  de `block-with-entities.dxf` — o defeito que motivou a fase deixa de existir.
  Os quatro degraus de desenho real continuam abrindo, sem perda de cobertura.
- **Fora de escopo:** substituir a renderização (K5) ou a leitura DWG (K6).
- **Depende de:** MT-K2-11.

---

## Ordem de execução

```text
A: 01 → 02
B: 03 → 04 → 05 → 06
C: 07 ─┬→ 08 → 09
       └ (07 depende só de 03; pode andar em paralelo a 04–06)
D: 10 → 11 → 12
```

## Riscos conhecidos

- **O DXF é grande e antigo.** A tentação é implementar tudo; o antídoto é o
  MT-K2-06 — o que não for compreendido é contado, não impede a abertura, e vira
  lista de trabalho futuro.
- **A escrita perde o que a leitura não entendeu.** Salvar um arquivo lido
  descarta o que o modelo não representa. Isso é destruição silenciosa de
  trabalho alheio e **precisa aparecer ao usuário** antes de `Salvar` sobrescrever
  original — a mesma preocupação que o ADR 0004 fixou para a edição por agentes.
  O MT-K2-11 deve tratar disso.
- **Determinismo é fácil de perder.** Ordem de iteração, formatação de ponto
  flutuante e locale entram no arquivo. Os testes de determinismo dos MT-K2-07 e
  08 existem para pegar isso cedo.
