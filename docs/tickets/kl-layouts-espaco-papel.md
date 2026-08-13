<!-- Caminho relativo: docs/tickets/kl-layouts-espaco-papel.md -->

# KL — Layouts de espaço-papel: micro-tickets

Quebra da fase **KL** do [ADR 0005](../adr/0005-layouts-de-espaco-papel.md), que
insere layouts entre K2 e K3 no faseamento do
[ADR 0003](../adr/0003-kernel-cad-proprio.md). Formato conforme a skill
`micro-ticket-planner`.

## Resultado esperado de KL

O NeoCAD abre, exibe, modela e grava layouts de espaço-papel como o AutoCAD.
Os **70% do acervo** que hoje abrem vazios ou sem a folha passam a abrir como
documento — que é o que o usuário emite, plota e envia ao cliente.

## Por que antes de K3

Medição em 1.989 desenhos reais: 8% não têm nada no espaço-modelo e abrem
mostrando nada; 63% têm os dois espaços povoados e abrem sem carimbo nem
viewports. Ferramenta que só lê espaço-modelo desenha, mas não emite documento.
K3 (edição 2D) é adiada uma fase em favor disso, por decisão registrada no ADR 0005.

## Ordem interna: o curto antes do certo

O bloco A entrega **exibição pelo upstream**, que já sabe desenhar layout
(`activeLayoutBtrId` tem leitura e escrita). O usuário vê suas pranchas sem
esperar o modelo próprio. Os blocos B e C constroem a representação própria, e o
D fecha a ida e volta em DXF.

Essa ordem tem um custo declarado: o bloco A produz código de fronteira que os
blocos seguintes vão reescrever. É deliberado — a alternativa é o usuário
esperar a fase inteira para ver a primeira prancha.

## O que KL **não** faz

- **Não renderiza por conta própria.** O desenho continua sendo do upstream até
  K5. O que ele não souber exibir permanece invisível ainda que o kernel o
  conheça, e isso precisa ser dito ao usuário, não escondido.
- **Não resolve `INSERT` em geometria.** Transformação de instância é trabalho de
  K3; o carimbo aparece porque o upstream desenha, não porque o kernel compõe.
- **Não amplia o conjunto de entidades modeladas** além de `Viewport`. Cotas,
  hachuras e splines continuam contadas e reportadas como não modeladas.

## Restrições que valem para todos os tickets

- O espaço de uma entidade é derivado do **bloco dono**, nunca de um campo
  paralelo (ADR 0005).
- Nada é descartado em silêncio: o que não for compreendido é contado e
  reportado, como já vale para a leitura de camadas.
- A escrita é determinística: o mesmo documento produz bytes idênticos
  (ADR 0004).
- Nenhum arquivo de origem confidencial entra no repositório (`AGENTS.md` §0.1);
  as fixtures são sintéticas.

---

## Bloco A — Ver a prancha (pelo upstream)

### MT-KL-01: Enumerar os layouts do documento aberto

- **Objetivo:** o adaptador do viewer passa a listar os layouts do documento —
  nome da aba, identificador do bloco associado e contagem de entidades —, com o
  espaço-modelo aparecendo como um item entre eles.
- **Arquivos no escopo:** `src/lib/viewer/neocad-viewer.ts`,
  `src/lib/types/cad.ts`.
- **Critério de aceite:** `pnpm check` e `pnpm test` verdes, com teste de unidade
  sobre objetos sintéticos no formato de `database.objects.layout`. A API do
  upstream é confirmada **em navegador** contra `e2e/fixtures/`, e o que for
  observado fica registrado no handoff — hoje ela é conhecida por inspeção do
  pacote, não por execução.
- **Fora de escopo:** exibir, trocar de layout, ou levar qualquer coisa ao kernel.
- **Depende de:** ADR-0005.

### MT-KL-02: Trocar o layout exibido

- **Objetivo:** o usuário escolhe a aba e o viewer passa a desenhá-la, escrevendo
  em `activeLayoutBtrId`. Abas na barra inferior, como no AutoCAD.
- **Arquivos no escopo:** `src/lib/viewer/neocad-viewer.ts`,
  `src/lib/components/workspace/LayoutTabs.svelte`, `src/routes/+page.svelte`.
- **Critério de aceite:** `pnpm test:e2e` demonstra a troca com uma fixture de
  dois layouts, conferindo que o conteúdo desenhado muda. `pnpm lint` verde.
- **Fora de escopo:** configuração de página; impressão; modelagem própria.
- **Depende de:** MT-KL-01.

### MT-KL-03: Dizer ao usuário onde o desenho está

- **Objetivo:** ao abrir, o NeoCAD informa quando o espaço-modelo está vazio e o
  conteúdo está em layout, e abre na aba que tem conteúdo em vez de numa tela
  vazia.
- **Arquivos no escopo:** `src/routes/+page.svelte`,
  `src/lib/services/cad-document.ts`.
- **Critério de aceite:** `pnpm test:e2e` cobre a fixture de espaço-modelo vazio:
  a mensagem nomeia o layout e o canvas não fica em branco. É o ticket que corrige
  o "0 entidade(s)" sem explicação que os 8% recebem hoje.
- **Fora de escopo:** ler o conteúdo do layout no kernel.
- **Depende de:** MT-KL-02.

---

## Bloco B — Modelar o layout

### MT-KL-04: Abrir a criação de blocos de nome reservado

- **Objetivo:** a `BlockTable` ganha via **interna** para criar registros de nome
  iniciado por `*`, como já existe para `*Model_Space`. A via pública continua
  recusando.
- **Arquivos no escopo:** `kernel/neocad-model/src/block.rs`,
  `kernel/neocad-model/src/lib.rs`.
- **Critério de aceite:** `cargo test -p neocad-model` cobre criação interna
  bem-sucedida e recusa pela via pública, com doctest `compile_fail` provando que
  a restrição é do compilador e não da documentação.
- **Fora de escopo:** `LayoutTable`; qualquer leitura de arquivo.
- **Depende de:** ADR-0005.

### MT-KL-05: Extrair `SymbolTable<T>`

- **Objetivo:** a máquina de índice por nome hoje triplicada em `LayerTable`,
  `BlockTable` e `TextStyleTable` vira uma estrutura genérica.
- **Arquivos no escopo:** `kernel/neocad-model/src/symbol_table.rs`,
  `kernel/neocad-model/src/{layer.rs,block.rs,text_style.rs,lib.rs}`.
- **Critério de aceite:** `make kernel-check` verde **sem alteração de nenhum
  teste existente** — é refatoração, e teste que precisa mudar denuncia mudança
  de comportamento.
- **Fora de escopo:** acrescentar capacidade nova a qualquer tabela.
- **Depende de:** MT-KL-04.

### MT-KL-06: `LayoutTable` no modelo

- **Objetivo:** quarta tabela de símbolos, guardando por layout o nome da aba, o
  bloco associado, a ordem das abas e a configuração de página (tamanho de papel,
  unidades, margens, escala e rotação de plotagem).
- **Arquivos no escopo:** `kernel/neocad-model/src/layout.rs`,
  `kernel/neocad-model/src/{document.rs,lib.rs}`.
- **Critério de aceite:** `cargo test -p neocad-model` cobre que remover um layout
  leva junto as entidades do bloco dele, que o espaço-modelo não é removível, e
  que a ordem das abas é determinística.
- **Fora de escopo:** viewports; leitura de arquivo.
- **Depende de:** MT-KL-05.

### MT-KL-07: `Viewport` como entidade

- **Objetivo:** nova geometria `Viewport` — janela no papel, centro e escala da
  vista no espaço-modelo, rotação e recorte.
- **Arquivos no escopo:** `kernel/neocad-model/src/entity.rs`,
  `kernel/neocad-model/src/geometry/viewport.rs`.
- **Critério de aceite:** `cargo test -p neocad-model` confere a caixa envolvente
  no papel e a conversão entre coordenada de papel e de modelo nos dois sentidos,
  com rotação diferente de zero.
- **Fora de escopo:** congelamento de camada por viewport; renderização.
- **Depende de:** MT-KL-06.

### MT-KL-08: Congelamento de camada por viewport

- **Objetivo:** cada viewport carrega o conjunto de camadas congeladas nela, que
  é o recurso do AutoCAD sem o qual uma prancha real mostra o que deveria estar
  oculto.
- **Arquivos no escopo:** `kernel/neocad-model/src/geometry/viewport.rs`,
  `kernel/neocad-model/src/layer.rs`.
- **Critério de aceite:** `cargo test -p neocad-model` demonstra que a
  visibilidade efetiva de uma camada **depende do viewport**, e que remover a
  camada limpa a referência em todos eles — referência pendurada aqui é entidade
  fantasma na prancha.
- **Fora de escopo:** aplicar isso na renderização.
- **Depende de:** MT-KL-07.

---

## Bloco C — Ler o layout do arquivo

### MT-KL-09: Ler as entidades por espaço

- **Objetivo:** a leitura de `ENTITIES` passa a escolher o bloco de destino pelo
  código `410` (aba) e, na falta dele, pelo `67`.
- **Arquivos no escopo:** `kernel/neocad-io/src/dxf/entities.rs`.
- **Critério de aceite:** `cargo test -p neocad-io` cobre entidade de modelo, de
  papel com `67`, de papel com `410` nomeando aba, e o caso de `410` apontando
  para aba inexistente — que **não pode** virar entidade perdida.
- **Fora de escopo:** interpretar `VIEWPORT`.
- **Depende de:** MT-KL-06, MT-K2-04.

### MT-KL-10: Ler os objetos `LAYOUT`

- **Objetivo:** a seção `OBJECTS` vira `LayoutTable`, com configuração de página e
  vínculo ao bloco.
- **Arquivos no escopo:** `kernel/neocad-io/src/dxf/objects.rs`.
- **Critério de aceite:** `cargo test -p neocad-io` lê uma fixture de dois layouts
  e confere nome de aba, ordem, tamanho de papel e bloco associado; layout sem
  bloco correspondente é **reportado**, não descartado.
- **Fora de escopo:** `VIEWPORT`.
- **Depende de:** MT-KL-09.

### MT-KL-11: Ler as entidades `VIEWPORT`

- **Objetivo:** `VIEWPORT` do espaço-papel vira a entidade do MT-KL-07, incluindo
  as camadas congeladas nela (código `331`).
- **Arquivos no escopo:** `kernel/neocad-io/src/dxf/entities.rs`.
- **Critério de aceite:** `cargo test -p neocad-io` confere janela, escala,
  rotação e a lista de camadas congeladas; o viewport de identificador `1`, que é
  a própria folha e não uma janela, é reconhecido e **não** vira janela.
- **Fora de escopo:** escrita.
- **Depende de:** MT-KL-10, MT-KL-08.

---

## Bloco D — Fechar a volta e chegar ao produto

### MT-KL-12: Escrever layouts, viewports e entidades de papel

- **Objetivo:** a escrita DXF reproduz os blocos `*Paper_Space*`, os objetos
  `LAYOUT` e os códigos `67`/`410` de cada entidade.
- **Arquivos no escopo:** `kernel/neocad-io/src/dxf/writer/{objects.rs,entities.rs}`.
- **Critério de aceite:** `cargo test -p neocad-io` confere a saída contra o
  esperado e o determinismo entre execuções.
- **Fora de escopo:** preservar o que a leitura não compreendeu.
- **Depende de:** MT-KL-11, MT-K2-08.

### MT-KL-13: Fechar a ida e volta com layout

- **Objetivo:** ler, escrever, reler e comparar um arquivo com layouts.
- **Arquivos no escopo:** `kernel/neocad-io/tests/round_trip_layout.rs`.
- **Critério de aceite:** o documento relido é igual ao original pela `PartialEq`
  semântica, **com a perda esperada declarada no teste**. É o ticket que sustenta
  a "total compatibilidade com AutoCAD" do ADR 0005: o que não sobreviver à volta
  fica escrito, não descoberto depois pelo usuário.
- **Fora de escopo:** preservação de conteúdo desconhecido.
- **Depende de:** MT-KL-12.

### MT-KL-14: Trocar a exibição de layout para o kernel

- **Objetivo:** as abas passam a ser alimentadas pela `LayoutTable` do kernel, e
  não pelo dicionário do upstream, que continua apenas desenhando.
- **Arquivos no escopo:** `src/lib/viewer/neocad-viewer.ts`,
  `src/lib/services/cad-document.ts`,
  `src/lib/components/workspace/LayoutTabs.svelte`.
- **Critério de aceite:** `pnpm test:e2e` continua passando **com os testes do
  MT-KL-02 intactos** — a fonte muda, o comportamento não. A contagem de
  entidades por layout que o kernel reporta bate com a fixture.
- **Fora de escopo:** substituir a renderização (K5).
- **Depende de:** MT-KL-13.

---

## Ordem de execução

```text
A: 01 → 02 → 03                     (valor observável cedo, pelo upstream)
B: 04 → 05 → 06 → 07 → 08           (modelo próprio; independe de A)
C: 09 → 10 → 11                     (09 também depende de MT-K2-04)
D: 12 → 13 → 14
```

Os blocos A e B não dependem um do outro e podem andar em paralelo.

## Riscos conhecidos

- **O upstream pode não exibir tudo.** `activeLayoutBtrId` é conhecido por
  inspeção do pacote, não por execução — o MT-KL-01 existe em parte para
  confirmar isso em navegador. Se a exibição falhar, o bloco A encolhe e o valor
  observável passa a depender de K5, o que muda a ordem mas não a decisão.
- **Congelamento por viewport alcança a `LayerTable`.** É a primeira vez que
  visibilidade de camada deixa de ser propriedade global do documento. Feito
  errado, contamina toda consulta de visibilidade do kernel.
- **`SymbolTable<T>` é refatoração de estrutura estabilizada.** O MT-KL-05 mexe
  em três tabelas que hoje funcionam. O critério "sem alterar nenhum teste
  existente" é o que impede a refatoração de virar mudança de comportamento
  disfarçada.
- **Escala de plotagem é onde a prancha mente.** Escala errada num viewport
  produz documento com cota certa e desenho errado — pior que não abrir. O
  MT-KL-07 exige conversão testada nos dois sentidos por isso.
