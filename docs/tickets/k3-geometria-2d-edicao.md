<!-- Caminho relativo: docs/tickets/k3-geometria-2d-edicao.md -->

# K3 — Geometria 2D e operações de edição: micro-tickets

Quebra da fase **K3** do [ADR 0003](../adr/0003-kernel-cad-proprio.md), adiada
uma fase pelo [ADR 0005](../adr/0005-layouts-de-espaco-papel.md) e assentada
sobre o [ADR 0006](../adr/0006-tolerancia-e-robustez-da-geometria-2d.md).
Formato conforme a skill `micro-ticket-planner`.

## Resultado esperado de K3

O NeoCAD deixa de ser um leitor-gravador e passa a **desenhar e editar**. Ao fim
da fase o kernel sabe onde as coisas se cruzam, o que está sob o cursor, onde o
cursor deve prender, e sabe mover, copiar, girar, espelhar, escalar, paralelar,
aparar, estender e concordar — tudo por transação reversível.

## Por que esta fase precede a de UI/UX

A fase seguinte é de **interface**, e interface de CAD é feita das operações
daqui. Um cursor que não prende em extremidade não é um cursor de CAD; uma
seleção que não sabe o que está sob o ponto não é uma seleção. Os tickets do
bloco B existem tanto por si quanto por serem o que a interface vai consumir —
daí virem antes das operações de edição, invertendo a ordem que o ADR 0003
sugere.

## O que K3 **não** faz

- **Não faz booleanas 2D.** O ADR 0003 as lista na fase, e elas saem daqui por
  decisão registrada abaixo. Booleana 2D robusta é recorte de polígono
  (Vatti, Greiner-Hormann) com todos os casos degenerados, e o que a consumiria
  é **hachura** — entidade que o modelo ainda não representa. Construir a
  operação antes da entidade que a usa é construir para ninguém. Volta quando
  hachura entrar.
- **Não desenha na tela.** Renderização é K5. As operações daqui são verificadas
  por teste no kernel, não por inspeção visual, e a interface que as aciona vem
  na fase de UI/UX com o desenho ainda a cargo do upstream.
- **Não resolve restrições.** Paramétrico é K4.
- **Não expande o conjunto de entidades modeladas.** Cotas, hachuras e splines
  seguem contadas e reportadas como não modeladas.
- **Não toca `INSERT`.** Compor a geometria de uma referência de bloco depende de
  transformação de instância, que é vizinha do bloco C mas puxa o modelo junto;
  fica para depois da UI/UX, quando houver caso de uso visível.

## Restrições que valem para todos os tickets

- **Toda comparação geométrica passa pelo `Tolerance`** (ADR 0006). Nenhum
  literal numérico solto, nenhuma constante nova de tolerância.
- **Toda operação nova é testada em duas magnitudes**, perto da origem e na
  ordem de `1e6` (ADR 0006). O teste de uma magnitude só não demonstra nada.
- **Decisão de lado ou sentido usa o predicado exato**, nunca epsilon.
- **Nenhuma mutação de documento fora de transação** — a reversibilidade é o que
  o ADR 0003 exige do kernel, e uma operação de edição que não desfaz é um
  defeito, não uma limitação.
- **`neocad-geometry` não depende de `neocad-model`.** A seta aponta num sentido
  só; geometria não sabe o que é entidade, camada ou documento.
- Nenhum arquivo de origem confidencial entra no repositório (`AGENTS.md` §0.1);
  as fixtures são sintéticas.

---

## Bloco A — Fundação (MT-K3-01 a MT-K3-03)

Antes de calcular é preciso ter com o quê. Hoje as primitivas moram na crate
errada e a polilinha não sabe representar arco.

### MT-K3-01: Mover as primitivas 2D para `neocad-geometry`

- **Objetivo:** `Line`, `Circle`, `Arc` e `Polyline` passam a ser definidas em
  `neocad-geometry`, com `neocad-model` reexportando-as para não quebrar quem as
  consome pelo nome atual. Nenhuma mudança de comportamento.
- **Por quê:** o ADR 0003 destina curvas a `neocad-geometry`, e elas estão em
  `neocad-model` por acidente de ordem de construção — K1 precisou de entidade
  antes de precisar de geometria. Deixá-las onde estão faria a crate de cálculo
  depender da crate de documento para falar de uma reta, invertendo a
  dependência que o ADR desenhou.
- **Arquivos no escopo:** `kernel/neocad-geometry/src/{lib.rs,curve.rs}`,
  `kernel/neocad-model/src/entity.rs`, `kernel/neocad-model/src/lib.rs`.
- **Critério de aceite:** `make kernel-check` verde **sem alteração em nenhum
  teste existente** — é o mesmo critério que valeu para a extração do
  `SymbolTable` no MT-KL-05, e pela mesma razão: refatoração de código
  estabilizado que muda teste não é refatoração.
- **Fora de escopo:** `Text` e `Viewport`, que não são curvas — `Text` carrega
  estilo e `Viewport` mostra outra coisa; ambos permanecem em `neocad-model`.
  Também fora: qualquer método novo.
- **Depende de:** nenhum.

### MT-K3-02: Introduzir o `Tolerance` e converter os epsilons existentes

- **Objetivo:** o tipo `Tolerance` do ADR 0006 existe em `neocad-geometry`, com
  tolerância efetiva adaptativa à magnitude, tolerância angular própria, e o
  predicado de orientação com filtro de erro e ramo exato.
- **Por quê:** é a peça que todos os tickets seguintes consomem, e o momento de
  escrevê-la é antes do primeiro consumidor, não depois do quarto.
- **Arquivos no escopo:** `kernel/neocad-geometry/src/{lib.rs,tolerance.rs,predicate.rs}`,
  e os módulos de teste de `neocad-geometry/src/point.rs` e
  `neocad-model/src/entity.rs`, que trocam seus epsilons pelo tipo comum.
- **Critério de aceite:** `make kernel-check` verde; o predicado de orientação
  tem teste de **consistência mútua** sobre trincas quase colineares — as três
  permutações não podem afirmar lados incompatíveis — e teste em coordenada da
  ordem de `1e6`, onde o determinante direto erra o sinal.
- **Fora de escopo:** tornar a tolerância configurável; qualquer operação
  geométrica que a use.
- **Depende de:** ADR-0006.

### MT-K3-03: Abaulamento (`bulge`) na polilinha

- **Objetivo:** a polilinha representa segmento em arco, e a leitura e a escrita
  DXF preservam o código de grupo `42`.
- **Por quê:** está anotado no próprio código desde K1 como dívida de K3. Sem
  abaulamento, toda polilinha com trecho curvo — que é a maioria das polilinhas
  de desenho real — é lida como se fosse toda reta, e a regravação **endireita o
  desenho em silêncio**. É perda de geometria que a ida-e-volta atual não acusa
  porque a leitura já a comete.
- **Arquivos no escopo:** `kernel/neocad-geometry/src/curve.rs`,
  `kernel/neocad-io/src/dxf/entities.rs`,
  `kernel/neocad-io/src/dxf/writer/entities.rs`,
  `kernel/neocad-io/tests/round_trip.rs`.
- **Critério de aceite:** `make kernel-check` verde; fixture sintética com
  polilinha de trecho curvo atinge ponto fixo na regravação; a perda que hoje
  existe sai do rol de perdas declaradas do MT-K2-09.
- **Fora de escopo:** largura de segmento (códigos `40`/`41`), que é atributo de
  exibição e não de forma.
- **Depende de:** MT-K3-01.

---

## Bloco B — O que a interface vai consultar (MT-K3-04 a MT-K3-07)

### MT-K3-04: Avaliação uniforme de curva

- **Objetivo:** uma interface comum às quatro primitivas — ponto em parâmetro,
  tangente, comprimento, ponto mais próximo de um ponto dado e `Aabb`.
- **Por quê:** sem isso, cada operação dos blocos seguintes reescreveria o mesmo
  `match` sobre quatro variantes, e cada cópia seria uma chance de as quatro
  divergirem. O ponto mais próximo é, além disso, o que sustenta o snap
  "próximo" e o teste de acerto.
- **Arquivos no escopo:** `kernel/neocad-geometry/src/curve.rs`.
- **Critério de aceite:** `make kernel-check` verde; para cada primitiva, o ponto
  mais próximo de um ponto sobre a curva é ele mesmo dentro da tolerância, nas
  duas magnitudes.
- **Fora de escopo:** parametrização por comprimento de arco.
- **Depende de:** MT-K3-02, MT-K3-03.

### MT-K3-05: Interseção entre primitivas

- **Objetivo:** reta×reta, reta×círculo, reta×arco, círculo×círculo, arco×arco e
  os pares envolvendo polilinha, com os casos de tangência e de sobreposição
  tratados explicitamente.
- **Por quê:** aparo, extensão, concordância e o snap de interseção são todos
  consumidores desta única operação. É o coração da fase.
- **Arquivos no escopo:** `kernel/neocad-geometry/src/intersect.rs`,
  `kernel/neocad-geometry/src/lib.rs`.
- **Critério de aceite:** `make kernel-check` verde, com teste dedicado para
  cada degenerescência — tangência, curvas coincidentes, segmentos colineares
  sobrepostos e o cruzamento que ocorre fora do trecho do arco — nas duas
  magnitudes.
- **Fora de escopo:** interseção de curva consigo mesma (autointerseção), que
  entra com a paralela, onde ela é o problema real.
- **Depende de:** MT-K3-04.

### MT-K3-06: Índice espacial e consulta por região

- **Objetivo:** consultar as entidades de um bloco por `Aabb` e por ponto com
  raio, sem varrer o documento inteiro.
- **Por quê:** varredura linear serve para mil entidades e não serve para as
  centenas de milhares de um desenho real. Cada movimento de cursor da fase de
  UI/UX é uma consulta destas.
- **Arquivos no escopo:** `kernel/neocad-geometry/src/spatial.rs`,
  `kernel/neocad-model/src/block.rs`.
- **Critério de aceite:** `make kernel-check` verde; teste que confronta o
  resultado do índice com o da varredura linear sobre um conjunto gerado,
  exigindo conjuntos idênticos — o índice pode ser mais rápido, nunca diferente.
- **Fora de escopo:** invalidação incremental do índice a cada edição; nesta
  fase ele é reconstruído, e a incrementalidade entra quando houver medição que
  a justifique.
- **Depende de:** MT-K3-04.

### MT-K3-07: Teste de acerto e prendimento (`osnap`)

- **Objetivo:** dado um ponto e um raio de captura, dizer que entidade está sob
  o cursor e a que ponto notável ele prende: extremidade, ponto médio, centro,
  quadrante, interseção, perpendicular, tangente e próximo.
- **Por quê:** é o gesto mais frequente de qualquer CAD, e o que distingue
  desenhar de rabiscar. Sem prendimento não há precisão, e sem precisão não há
  desenho de engenharia.
- **Arquivos no escopo:** `kernel/neocad-geometry/src/snap.rs`.
- **Critério de aceite:** `make kernel-check` verde; a prioridade entre modos
  concorrentes dentro do mesmo raio é **determinística e testada** — dois pontos
  notáveis à mesma distância não podem produzir resultado que dependa da ordem
  de iteração.
- **Fora de escopo:** rastreamento polar e prendimento a referência temporária,
  que são comportamento de interface e pertencem à fase de UI/UX.
- **Depende de:** MT-K3-05, MT-K3-06.

---

## Bloco C — Edição (MT-K3-08 a MT-K3-11)

### MT-K3-08: Transformações de entidade

- **Objetivo:** mover, copiar, girar, espelhar e escalar entidades, cada uma
  como comando transacional reversível.
- **Por quê:** são as cinco operações que respondem pela maior parte do tempo de
  quem edita um desenho, e todas as quatro seguintes pressupõem que geometria
  possa ser transformada.
- **Arquivos no escopo:** `kernel/neocad-geometry/src/transform.rs`,
  `kernel/neocad-model/src/entity.rs`,
  `kernel/neocad-transaction/src/commands.rs`.
- **Critério de aceite:** `make kernel-check` verde; para cada operação, desfazer
  restaura o documento a bytes idênticos aos de antes, verificado pela gravação
  determinística do ADR 0004 — que é um teste mais forte que comparar campo a
  campo, e reaproveita o que K2 já construiu.
- **Fora de escopo:** matriz e arranjo (`array`), que são repetição de cópia e
  ficam para a fase de UI/UX, onde o valor está na interação.
- **Depende de:** MT-K3-04.

### MT-K3-09: Paralela (`offset`)

- **Objetivo:** paralela a distância dada, para reta, arco, círculo e polilinha,
  com o lado escolhido por um ponto.
- **Por quê:** é a operação de que o desenho técnico mais depois depende —
  espessura de parede, faixa de servidão, contorno de peça.
- **Arquivos no escopo:** `kernel/neocad-geometry/src/offset.rs`,
  `kernel/neocad-transaction/src/commands.rs`.
- **Critério de aceite:** `make kernel-check` verde; teste do caso que define a
  operação: polilinha cuja paralela **se autointersecta**, em que o resultado
  correto é a curva aparada e não a ingênua vértice-a-vértice; e do arco cujo
  raio a paralela anularia, em que a operação recusa em vez de emitir raio
  negativo.
- **Fora de escopo:** paralela de curva com abaulamento variável.
- **Depende de:** MT-K3-05.

### MT-K3-10: Aparar e estender (`trim`/`extend`)

- **Objetivo:** aparar uma curva contra arestas de corte e estender uma curva
  até uma fronteira, escolhendo o trecho pelo ponto que o usuário indica.
- **Por quê:** com a interseção pronta, o que resta é decidir **qual pedaço
  fica** — e é justamente essa escolha que faz a operação parecer óbvia ao
  desenhista e ser sutil no código.
- **Arquivos no escopo:** `kernel/neocad-geometry/src/trim.rs`,
  `kernel/neocad-transaction/src/commands.rs`.
- **Critério de aceite:** `make kernel-check` verde; teste do aparo que parte uma
  curva em **duas**, do aparo contra várias arestas simultâneas e da extensão que
  não alcança fronteira alguma, que deve recusar sem alterar o documento.
- **Fora de escopo:** aparo por aresta implícita (todas as entidades visíveis
  como corte), que é conveniência de interface.
- **Depende de:** MT-K3-05, MT-K3-08.

### MT-K3-11: Concordância e chanfro (`fillet`/`chamfer`)

- **Objetivo:** arco de concordância de raio dado e chanfro de distâncias dadas
  entre duas curvas, aparando-as ao resultado.
- **Por quê:** fecha o conjunto de edição que um desenho de engenharia exige, e
  é o consumidor mais severo do ADR 0006: a concordância decide tangência, que é
  precisamente onde comparar ângulo em vez de distância dá errado.
- **Arquivos no escopo:** `kernel/neocad-geometry/src/fillet.rs`,
  `kernel/neocad-transaction/src/commands.rs`.
- **Critério de aceite:** `make kernel-check` verde; teste da concordância de
  raio zero, que deve equivaler a estender ao vértice; do raio grande demais para
  caber, que deve recusar; e da escolha entre as quatro soluções possíveis, que
  deve ser governada pelos pontos indicados e não pela ordem dos argumentos.
- **Fora de escopo:** concordância entre polilinha e polilinha em um só gesto.
- **Depende de:** MT-K3-05, MT-K3-10.

---

## Bloco D — Chegada ao produto (MT-K3-12)

### MT-K3-12: Expor consulta e edição na fachada WebAssembly

- **Objetivo:** `CadSession` ganha as consultas do bloco B e os comandos do bloco
  C, com desfazer e refazer atravessando a fronteira.
- **Por quê:** os onze tickets anteriores são invisíveis ao usuário até este. É a
  mesma forma dos blocos finais de K2 e KL, e existe para que a fase termine em
  algo que a fase de UI/UX possa acionar em vez de em algo que só o teste vê.
- **Arquivos no escopo:** `kernel/neocad-wasm/src/lib.rs`,
  `src/lib/types/cad.ts`, `src/lib/services/cad-document.ts`.
- **Critério de aceite:** `make kernel-check`, `pnpm lint`, `pnpm test` e
  `make e2e` verdes; o tamanho do `.wasm` é medido e registrado, como no
  MT-K2-10 — a fase acrescenta cálculo, e cálculo pesa.
- **Fora de escopo:** qualquer elemento de interface. Botão, cursor e barra de
  ferramentas são a fase seguinte; aqui só se abre a porta.
- **Depende de:** MT-K3-07, MT-K3-11.

---

## Sequência

```
A: 01 → 02 → 03
B: 04 → 05 ⇘
        06 ⇒ 07
C: 08 → 09
   10 → 11
D: 12
```

MT-K3-01 e MT-K3-02 são independentes entre si e podem trocar de ordem; os
demais seguem as dependências declaradas. O bloco C só começa depois de o bloco
B fechar, porque aparo, paralela e concordância consomem interseção, e não o
contrário.
