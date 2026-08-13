<!-- Caminho relativo: docs/adr/0005-layouts-de-espaco-papel.md -->

# ADR 0005: Layouts de espaço-papel como conceito de primeira classe do kernel

- **Status:** Accepted <!-- Proposed | Accepted | Deprecated | Superseded by ADR-MMMM -->
- **Data:** 2026-08-12
- **Decisores:** Iago Leal
- **Tags:** kernel, dxf, layout, espaço-papel, autocad, roadmap

## Contexto

O NeoCAD lê hoje **apenas o espaço-modelo**. A extração pega
`database.tables.blockTable.modelSpace` e nada mais — tudo o que estiver em
layout de espaço-papel é invisível para o kernel.

Uma varredura do acervo real mediu o custo disso. De 2.396 desenhos DWG lidos,
descontadas 407 bibliotecas de ferragens e símbolos (arquivos cuja geometria vive
toda em definição de bloco, e que não são desenhos), sobram 1.989:

| grupo                 | nº   | %   | como o NeoCAD abre hoje |
| --------------------- | ---- | --- | ----------------------- |
| nada no espaço-modelo | 153  | 8%  | mostrando **nada**      |
| ambos os espaços      | 1244 | 63% | **sem a folha**         |
| só espaço-modelo      | 592  | 30% | corretamente            |

**70% dos desenhos têm conteúdo no espaço-papel.** O corte por "espaço-modelo
vazio" isoladamente engana: são só 8%. Os 63% restantes também dependem de
layout, porque é lá que estão o carimbo e as viewports que compõem a prancha —
abri-los só pelo modelo mostra o desenho sem a folha.

O usuário declarou o requisito com dois qualificadores que fixam o nível:
**"extrema importância"** e **"total compatibilidade com AutoCAD"**, incluindo
ler os layouts do AutoCAD, "pois é exatamente isto que gera os documentos". Em
projeto de linha de transmissão o entregável não é o desenho no espaço-modelo — é
a prancha composta, com carimbo, escalas e viewports. Uma ferramenta que só lê o
espaço-modelo **desenha, mas não emite documento**.

Nem o [ADR 0003](0003-kernel-cad-proprio.md), que fixou as fases K1–K9, nem o
[ADR 0004](0004-interface-para-agentes-de-ia.md), que estendeu K2 para incluir
leitura DXF, previram layout em lugar nenhum. A omissão não foi deliberada:
ambos foram escritos antes de existir medição de acervo.

**Por que decidir agora, e não depois.** O MT-K2-04 é o ticket que lê as
entidades da seção `ENTITIES`, e é exatamente onde o espaço de cada entidade é
preservado ou descartado. No DXF as entidades de papel não vivem em lugar
separado: estão na mesma seção, marcadas pelo código `67` (`1` = papel) e pelo
`410` (aba de layout). Preservar essa informação custa dois códigos de grupo;
descartá-la obriga a reescrever o leitor depois. A janela para decidir barato é
antes daquele ticket.

**Um caminho curto existe.** O upstream `@mlightcad/cad-simple-viewer` já expõe
`activeLayoutBtrId` com leitura e escrita, mais `activeLayout` e o dicionário
`database.objects.layout`. Ou seja: **a camada de desenho de terceiros já sabe
exibir um layout**, e o que falta para o usuário ver sua prancha é do lado do
NeoCAD. Isso permite entregar valor observável antes de a leitura nativa existir,
pelo mesmo padrão que o K1 usou — upstream lê e desenha, kernel é a verdade sobre
o que existe.

## Decisão

Fica acordado que **layout de espaço-papel é conceito de primeira classe do
kernel do NeoCAD**, e não detalhe de renderização nem recurso de exportação.

**1. Nova fase KL, entre K2 e K3.** O faseamento do ADR 0003 passa a ser
K1 → K2 → **KL** → K3 → … → K9. A fase entrega leitura, modelagem, exibição e
escrita de layouts. Esta é uma emenda ao ADR 0003, no mesmo espírito da que o ADR
0004 já fez ao escopo de K2.

**2. Layout é modelado como o AutoCAD o modela: um _block record_ mais
metadados.** O espaço-modelo e cada layout são registros de bloco —
`*Model_Space`, `*Paper_Space`, `*Paper_Space0` —, e uma entidade pertence a
exatamente um deles. A `BlockTable` construída em K1 já tem essa forma: bloco raiz
indestrutível, lista ordenada de entidades por bloco, nomes iniciados por `*`
reservados. A fase reaproveita a estrutura em vez de criar outra.

**3. O espaço não vira atributo da entidade.** O bloco dono já diz onde a entidade
está; duplicar essa informação num campo criaria duas fontes de verdade que
podem divergir, e espalharia o conceito de "espaço" por todo o kernel. Os códigos
`67` e `410` do DXF são usados na **leitura**, para escolher o bloco de destino, e
na **escrita**, para reproduzi-los — não viram estado do modelo.

**4. `LayoutTable` como quarta tabela de símbolos.** Guarda, por layout, o nome da
aba, o bloco associado, a ordem das abas e a configuração de página (tamanho de
papel, unidades, margens, escala e rotação de plotagem). Sua criação é o gatilho
da extração de `SymbolTable<T>` já registrada como dívida — a quarta tabela é
exatamente o limite que aquela dívida marcou.

**5. Viewport é entidade.** Carrega a janela no papel, o centro e a escala da
vista no espaço-modelo, a rotação, o recorte e o **congelamento de camada por
viewport**. Este último é recurso que "compatibilidade total com AutoCAD" exige e
que alcança a `LayerTable`, hoje sem noção de estado por viewport.

**6. O caminho curto vem primeiro.** A fase começa entregando enumeração e troca
de layout **pelo upstream**, que já sabe desenhá-los, para que o usuário abra suas
pranchas antes de a leitura nativa existir. A modelagem própria vem em seguida.

**7. Compatibilidade é critério de aceite, não aspiração.** Um arquivo com layouts
lido e reescrito pelo NeoCAD preserva os objetos de layout, e o que a leitura não
compreender é **contado e reportado**, nunca descartado em silêncio.

## Consequências

- **Impacto positivo:** os 70% do acervo que hoje abrem vazios ou sem a folha
  passam a abrir como documento. O modelo de blocos do K1 é reaproveitado em vez
  de duplicado. O caminho curto pelo upstream entrega valor observável cedo, sem
  esperar a leitura nativa.
- **Impacto negativo:** a fase amplia o modelo — `LayoutTable`, `Viewport`,
  estado de camada por viewport — antes de K3, que era a próxima ampliação
  prevista. O congelamento por viewport toca a `LayerTable`, estrutura já
  estabilizada. A renderização **própria** da prancha continua dependendo de K5;
  até lá o desenho é do upstream, e o que ele não souber exibir permanece
  invisível ainda que o kernel o conheça.
- **Trade-offs aceitos:** K3 (geometria 2D e operações de edição) é adiada em uma
  fase, o que atrasa as ferramentas de edição em favor de conseguir emitir
  documento. A criação de blocos com nome iniciado por `*` — hoje recusada pela
  validação de nomes, e por bom motivo — passa a exigir uma via interna
  controlada, análoga à que cria `*Model_Space`; é uma abertura real na proteção
  daquele espaço de nomes, restrita à crate do modelo.

## Diretriz de Conformidade de Código

- **Proibido:** descartar, na leitura de arquivo, a informação de a qual espaço ou
  layout uma entidade pertence.
- **Proibido:** gravar arquivo perdendo em silêncio objetos de layout que a
  leitura tenha compreendido; a perda, quando inevitável, é contada e reportada
  ao usuário antes de sobrescrever original.
- **Proibido:** tratar layout como detalhe da camada de renderização, ou
  representá-lo fora do modelo do documento.
- **Proibido:** criar bloco com nome iniciado por `*` por via pública; nomes
  reservados são criados apenas pela via interna da crate do modelo.
- **Obrigatório:** toda entidade pertence a exatamente um registro de bloco, e o
  espaço a que ela pertence é derivado desse vínculo, nunca de um campo paralelo.
- **Obrigatório:** layouts são representados como registro de bloco mais
  metadados na `LayoutTable`, espelhando a estrutura do AutoCAD.
- **Obrigatório:** toda alteração de layout passa pela transação do command stack,
  como qualquer outra mutação do documento (ADR 0003).

> Qualquer desvio desta regra viola as diretrizes de conformidade arquitetural do
> projeto e deve ser reportado para revisão antes de prosseguir.
