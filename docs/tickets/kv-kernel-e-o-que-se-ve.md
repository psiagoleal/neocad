<!-- Caminho relativo: docs/tickets/kv-kernel-e-o-que-se-ve.md -->

# KV — O kernel é o que se vê: micro-tickets

Fase KV do [ADR 0007](../adr/0007-fase-de-prancha-de-engenharia.md), aceita contra a
[spec 07](../specs/07-kernel-e-o-que-se-ve.md).

## Por que a fase começa por medir

O mecanismo — espelhar o kernel no modelo do upstream ou desenhar a partir do kernel —
é decisão de arquitetura, e a skill `micro-ticket-planner` não deixa abrir ticket que
dependa de decisão não registrada. Por isso o primeiro bloco **prova o defeito e mede as
duas opções**; os tickets de implementação são escritos depois do ADR 0008.

## Bloco A — Prova e medida

### MT-KV-01: E2E que prova a perda da linha desenhada pelo upstream

- **Objetivo:** E2E que desenha uma linha pela barra de comandos atual, salva, reabre e
  confere a linha — **falhando hoje**, pelo motivo certo (spec 07, V4; achado C4).
- **Arquivos no escopo:** `e2e/kernel-view.e2e.ts` (novo).
- **Critério de aceite:** o teste falha, e a saída citada no handoff mostra a ausência da
  linha no arquivo reaberto; fica marcado `test.fail()` até KV fechar, para não quebrar a
  CI.
- **Fora de escopo:** corrigir.
- **Depende de:** nenhum.

### MT-KV-02: Gerador de desenho sintético grande

- **Objetivo:** script que gera DXF sintético com N entidades das primitivas modeladas,
  distribuídas em camadas e em dois layouts, para medir abertura e edição sem arquivo de
  terceiros (spec 00, T10).
- **Arquivos no escopo:** `scripts/gerar-dxf-sintetico.mjs` (novo).
- **Critério de aceite:** gera 100 mil entidades; o arquivo abre no NeoCAD e o kernel
  informa a contagem exata.
- **Fora de escopo:** entidades não modeladas.
- **Depende de:** nenhum.

### MT-KV-03: Spike — espelhar o kernel no modelo do upstream

- **Objetivo:** protótipo descartável que aplica ao `data-model` do upstream cada mudança
  do kernel (criar, alterar, remover, desfazer) e mede: tempo de abertura e de refletir uma
  edição no desenho de 100 mil entidades, e quantos tipos do kernel o upstream representa
  fielmente.
- **Arquivos no escopo:** ramo de spike, fora de `master`; relatório em
  `docs/spikes/kv-espelhamento.md` (novo).
- **Critério de aceite:** relatório com os números contra as metas da spec 07 e a lista
  do que não fecha.
- **Fora de escopo:** código de produção.
- **Depende de:** MT-KV-02.

### MT-KV-04: Spike — desenhar a partir do kernel

- **Objetivo:** protótipo descartável de desenho 2D direto do kernel (linhas, arcos,
  círculos, polilinhas, texto simples) sobre o mesmo canvas, com pan e zoom, medindo os
  mesmos números do MT-KV-03.
- **Arquivos no escopo:** ramo de spike; relatório em `docs/spikes/kv-render-kernel.md`
  (novo).
- **Critério de aceite:** relatório com os mesmos números e o custo estimado de chegar à
  paridade visual com o upstream nas entidades modeladas.
- **Fora de escopo:** código de produção; seleção e alças (K5).
- **Depende de:** MT-KV-02.

### MT-KV-05: ADR 0008 — mecanismo de KV

- **Objetivo:** ADR que escolhe o mecanismo com base nos dois relatórios e fixa os
  não-funcionais definitivos na spec 00.
- **Arquivos no escopo:** `docs/adr/0008-*.md`, `docs/adr/README.md`,
  `docs/specs/00-transversal.md`, `docs/specs/07-kernel-e-o-que-se-ve.md`.
- **Critério de aceite:** ADR aceito pelo mantenedor; spec 07 com os não-funcionais em
  EARS e prova.
- **Fora de escopo:** implementar.
- **Depende de:** MT-KV-03, MT-KV-04.

## Bloco B — Implementação

Escrito depois do [ADR 0008](../adr/0008-tela-desenhada-a-partir-do-kernel.md): a tela é
desenhada a partir do kernel, pela GPU (WebGL2) com reserva em canvas 2D; o kernel
prepara a geometria para a tela e o TypeScript só a repassa. Os não-funcionais são N1–N5
da [spec 00](../specs/00-transversal.md).

Ordem: a fachada (06) e a prova de desempenho (07) primeiro, para que o desenhista nasça
contra um teste que falha; depois o preparo no kernel (08), a escolha da camada da GPU
(09) e o desenhista (10 a 12); por fim o upstream reduzido (13), os comandos (14) e a
verificação por fabricante (15).

### MT-KV-06: Fachada informa o que cada operação mudou e consulta pelo id

- **Objetivo:** toda operação de edição da fachada devolve os ids criados, alterados e
  removidos, inclusive desfazer e refazer; a fachada ganha a consulta de entidades por uma
  lista de ids.
- **Arquivos no escopo:** `kernel/neocad-wasm/src/`, `src/lib/services/cad-document.ts`,
  `src/lib/services/cad-document.spec.ts`.
- **Critério de aceite:** `cargo test --manifest-path kernel/Cargo.toml -p neocad-wasm` e
  `pnpm test -- cad-document` cobrindo criar, mover, apagar, desfazer e refazer; tempo da
  consulta de um id no desenho de 100 mil entidades medido e citado no handoff.
- **Fora de escopo:** desenhar.
- **Depende de:** MT-KV-05.

### MT-KV-07: Prova local de desempenho (N1–N3)

- **Objetivo:** E2E que gera o desenho sintético de 100 mil entidades, abre, edita e faz
  pan e zoom, e confere N1, N2 e N3 contra a tela própria. Pulado sem
  `NEOCAD_SINTETICO_N`, como o teste de escala do MT-KV-02.
- **Arquivos no escopo:** `e2e/desempenho.e2e.ts` (novo).
- **Critério de aceite:** com `NEOCAD_SINTETICO_N=100000`, o teste falha hoje pelo motivo
  certo (a tela própria ainda não existe) e a saída é citada no handoff; na CI fica
  pulado.
- **Fora de escopo:** desenhista.
- **Depende de:** MT-KV-05.

### MT-KV-08: Preparo para a tela no kernel

- **Objetivo:** o kernel transforma as entidades modeladas em blocos contínuos de
  `Float32Array` por espaço e camada (segmentos de reta, com arcos, círculos e polilinhas
  divididos pelo erro de corda da vista), mantém o índice espacial para o recorte pela
  vista e atualiza só o que os ids da operação indicam.
- **Arquivos no escopo:** módulo novo em `kernel/neocad-geometry/src/` (segmentação) e em
  `kernel/neocad-wasm/src/` (blocos e índice), `src/lib/services/cad-document.ts`.
- **Critério de aceite:** `cargo test` com erro de corda dentro da tolerância do ADR 0006
  para arco e círculo; tempo de preparar as 100 mil entidades e de atualizar uma medido e
  citado no handoff.
- **Fora de escopo:** texto (MT-KV-11); desenhar.
- **Depende de:** MT-KV-06.

### MT-KV-09: Escolher a camada da GPU — WebGL2 em TypeScript ou `wgpu` em Rust

- **Objetivo:** protótipo descartável das duas camadas sobre os blocos do MT-KV-08,
  medindo abertura, edição, pan e zoom (N1–N3), tamanho do WASM e esforço do texto.
- **Arquivos no escopo:** ramo de spike; relatório em `docs/spikes/kv-camada-gpu.md`
  (novo).
- **Critério de aceite:** relatório com os números e a escolha, citada no MT-KV-10.
- **Fora de escopo:** código de produção.
- **Depende de:** MT-KV-08.
- **Resultado:** [WebGL2 em TypeScript](../spikes/kv-camada-gpu.md); `wgpu` levaria o
  `.wasm` a 2 042 252 bytes sem ganho medido.

### MT-KV-10: Desenhista pela GPU com reserva em canvas 2D

- **Objetivo:** desenhar retas, arcos, círculos e polilinhas do kernel, nos dois espaços,
  com pan e zoom pela matriz da vista; cair sozinho no canvas 2D sem WebGL2.
- **Arquivos no escopo:** `src/lib/render/` (novo), `src/lib/viewer/neocad-viewer.ts`,
  `e2e/kernel-document.e2e.ts`, `e2e/kernel-view.e2e.ts`.
- **Critério de aceite:** spec 07, V1 (sem texto); spec 00, N1, N3 e N4, com o
  MT-KV-07 passando localmente e o resultado citado no PR.
- **Fora de escopo:** texto; edição.
- **Depende de:** MT-KV-07, MT-KV-09.
- **Resultado (2026-11-01):** `src/lib/render/` (vista, cor, WebGL2, canvas 2D e o
  controlador) e `KernelScreen.svelte`; o kernel ganhou `screenExtents` e
  `screenEntityCount`. V1 sem texto (modelo e cada layout, por abas) e N4 provados em
  `e2e/kernel-view.e2e.ts`. Com Chrome em janela na RTX 3070, 100 mil entidades, três
  rodadas: N1 3,57 / 3,79 / 4,10 s (meta 5 s); N3 16,7 ms nas três (meta 33,3 ms). A
  medida achou um defeito: a camada de carregamento do upstream (`z-index: 9999`)
  cobria a tela própria e engolia o mouse até o upstream terminar; a área do upstream
  passou a ter camada de empilhamento própria, abaixo da tela. O teste de N3 recusa
  renderizador em software. A reserva sem WebGL2 vale só para a tela própria: sem
  WebGL2 na página, o upstream não monta e a abertura não habilita, até o MT-KV-13.

### MT-KV-11: Texto do kernel na tela

- **Objetivo:** posição dos glifos calculada no kernel, desenho pelo caminho da GPU e
  pelo de reserva.
- **Arquivos no escopo:** a definir no MT-KV-09, conforme a camada escolhida.
- **Critério de aceite:** spec 07, V1 completo; N1 e N3 mantidos; em
  `e2e/desempenho.e2e.ts`, a espera de N1 volta a ser `contagem.modelo` (hoje desconta
  os textos).
- **Fora de escopo:** MTEXT e estilos de texto (fase KP).
- **Depende de:** MT-KV-10.
- **Resultado (2026-11-01):** fonte de traço Hershey Roman Simplex embutida em
  `neocad-geometry` ([ADR 0010](../adr/0010-texto-da-tela-em-fonte-de-traco.md)), com
  acentos compostos e os códigos `%%`; o texto vira segmentos no bloco da camada, então o
  desenhista não mudou. Provas: `stroke_font` e `tela` em `cargo test`; V1 com texto em
  `e2e/kernel-view.e2e.ts` (fixture `with-text.dxf`), visto falhar com o texto desligado
  (1 em vez de 3); amostra conferida em captura. Com Chrome em janela na RTX 3070, 100
  mil entidades e a espera de N1 de volta a `contagem.modelo`: N1 2,79 / 2,89 / 2,92 s,
  N2 12,9–15,4 ms, N3 16,7 ms. `.wasm` 479 595 bytes (+14 872).

### MT-KV-12: Refletir edição, desfazer e refazer

- **Objetivo:** a tela atualiza só o que os ids da operação indicam.
- **Arquivos no escopo:** `src/lib/render/`, `e2e/kernel-view.e2e.ts`.
- **Critério de aceite:** spec 07, V2 e V3; spec 00, N2, pelo MT-KV-07.
- **Fora de escopo:** comandos digitados.
- **Depende de:** MT-KV-10.
- **Resultado (2026-11-01):** a tela já buscava só os blocos que `updateScreen` aponta;
  faltavam a sonda (`src/lib/services/sonda.ts`, ligada em `+page.svelte` e sempre
  presente no build dos testes E2E) e o atributo `data-caixa`. V2 e V3 provados em
  `e2e/kernel-view.e2e.ts`; a sonda fica fora do build de produção (conferido em
  `build/`). N2 com Chrome em janela na RTX 3070, 100 mil entidades: 11,3 / 14,3 /
  11,2 ms (meta 100 ms). O escopo passou de `src/lib/render/` para incluir a sonda, a
  página e `playwright.config.ts`.

### MT-KV-13: Upstream só para o que o kernel não modela

Fatiado em 2026-11-01, ao escrever o aceite: são três objetivos com riscos diferentes, e
tirar a barra do upstream antes da linha de comando própria (MT-KV-14) deixaria o usuário
sem nenhum comando de desenho. A API do upstream dá o necessário: `AcTrView2d.removeEntity`
tira entidades da cena dele, e `center`/`zoomTo` permitem seguir a vista da tela própria.

### MT-KV-13a: Upstream desenha só o não modelado, abaixo da tela e na mesma vista

- **Objetivo:** com o DXF lido pelo kernel, o upstream tira da cena dele as entidades que
  o kernel modela e desenha o resto numa camada abaixo da tela própria, que passa a ter
  fundo transparente; pan e zoom da tela própria movem a vista do upstream junto.
- **Arquivos no escopo:** `src/lib/viewer/neocad-viewer.ts`,
  `src/lib/render/tela-do-kernel.ts`, `src/routes/+page.svelte`,
  `e2e/kernel-view.e2e.ts`.
- **Critério de aceite:** com `with-unsupported.dxf`, a entidade não modelada aparece e
  acompanha pan e zoom (conferido pela vista do upstream exposta em atributo); as
  modeladas não são desenhadas duas vezes; N1 e N3 mantidos.
- **Fora de escopo:** edição; barra de comandos.
- **Depende de:** MT-KV-12.
- **Resultado (2026-11-01):** o kernel diz quais tipos DXF a tela dele desenha
  (`SCREEN_ENTITY_TYPES` em `tela.rs`: LINE, CIRCLE, ARC, LWPOLYLINE, POLYLINE e TEXT;
  `screenEntityTypes` na fachada; `CadDocument.getScreenEntityTypes`). Durante a abertura,
  o adaptador do upstream não entrega à cena dele as entidades desses tipos; depois da
  abertura o filtro sai, para o que a barra do upstream desenhar continuar aparecendo até
  o MT-KV-13b. O nome do tipo vem do upstream (`dxfTypeName`): a SOLID, por exemplo, é
  `TRACE` lá, e por isso a lista é do que o kernel desenha, não do que ele recusa.
  A tela própria fica transparente no espaço-modelo e avisa a vista a cada quadro; o
  adaptador põe o centro e mede o zoom 1 antes de aplicar a escala, porque a câmera do
  upstream não fala em pixels, e reaplica quando o upstream muda a vista sozinho
  (`viewChanged`, `viewResize`). O fundo inicial do upstream passou ao do tema escuro.
  Provas: `cada_tipo_da_lista_vira_tracos_e_o_resto_fica_fora` (Rust),
  `vista-do-upstream.spec.ts`, `canvas2d.spec.ts` (fundo transparente),
  `cad-document-sessao.spec.ts` e o e2e "o upstream desenha só o que o kernel não modela"
  (visto falhar antes: `data-entidades` do upstream ausente). Vista conferida em captura:
  os vértices da SOLID caem sobre a LINE antes e depois de pan e zoom. RTX 3070, 100 mil
  entidades: N1 2,88 s, N2 12,3 ms, N3 16,7 ms. Fora do escopo do ticket, tocados:
  `tela.rs`, `lib.rs` da fachada, `cad-document.ts`, `desenhista.ts`, `webgl2.ts`,
  `canvas2d.ts` e `ViewerDropzone.svelte` (o atributo `data-tela="upstream"`).
  Limites: um `INSERT` segue inteiro com o upstream (o kernel não modela referência de
  bloco), inclusive as linhas do bloco; num layout, o upstream fica escondido sob a tela
  opaca.

### MT-KV-13b: Barra de comandos e edição do upstream saem (V5)

- **Objetivo:** a barra de comandos e todo comando de edição do upstream deixam de ser
  oferecidos; o catálogo lista só comando que edita pelo kernel.
- **Arquivos no escopo:** `src/lib/viewer/neocad-viewer.ts`,
  `src/lib/services/cad-commands.ts`, `e2e/kernel-view.e2e.ts` (o V4 passa a usar a
  linha de comando própria).
- **Critério de aceite:** spec 07, V5 (`pnpm test -- cad-command-catalog`).
- **Fora de escopo:** linha de comando própria.
- **Depende de:** MT-KV-14b (concluído).

### MT-KV-13c: Aviso das entidades que a tela não exibe (V6)

- **Objetivo:** quando o caminho de exibição recusa uma entidade do kernel, a área de
  mensagens informa o tipo e a quantidade das não exibidas.
- **Arquivos no escopo:** `src/lib/render/tela-do-kernel.ts`, teste de unidade novo.
- **Critério de aceite:** spec 07, V6.
- **Fora de escopo:** desenhar as recusadas.
- **Depende de:** MT-KV-10.
- **Resultado (2026-11-01):** a tela do kernel guarda as entidades que não viram
  segmentos e `Screen::refused` as conta por tipo DXF, por espaço, só em camada visível
  (`screenRefused` na fachada, `CadDocument.getScreenRefused`). Hoje são a janela de
  layout e o texto sem traço. A `TelaDoKernel` avisa por `avisarCom` ao mostrar o espaço
  e quando uma edição muda a contagem, sem repetir o mesmo aviso; a página manda o aviso
  à área de mensagens. Provas: `o_que_nao_vira_traco_e_contado_por_tipo_e_espaco` e
  `a_janela_de_layout_lida_e_contada_como_nao_desenhada` (Rust), `tela-do-kernel.spec.ts`
  (recusa simulada, a prova da spec), `cad-document-sessao.spec.ts` e o e2e V6 com a
  fixture nova `with-viewport.dxf`, todos vistos falhar antes. A primeira versão somava
  14 914 bytes ao `.wasm` por trazer dois mapas de tipo novo; reaproveitando o mapa que a
  tela já usava e contando num vetor, ficou em +5 719 (485 816 bytes). N1 2,83 s, N2
  12,2 ms, N3 16,7 ms. Fora da lista do ticket, tocados: `tela.rs`, `lib.rs` da fachada,
  `cad-document.ts` e `+page.svelte`.

### MT-KV-14: Comandos de edição pelo kernel na linha de comando própria

Fatiado em 2026-11-02, ao escrever o aceite: o ticket pedia o motor de comandos, a entrada
de pontos, o componente visível, a seleção na tela e os comandos de edição do K3 de uma
vez. São cinco objetivos, cada um com prova própria. O `LINE` ganhou critérios na
[spec 10](../specs/10-desenho-basico.md) (D1–D6).

### MT-KV-14a: Motor de comandos, entrada de pontos e LINE

- **Objetivo:** módulo puro que recebe o texto confirmado e conduz o comando em curso por
  prompts, com o `LINE` como primeiro comando, despachado pelo `CadDocument`.
- **Arquivos no escopo:** `src/lib/command-line/motor.ts`, `pontos.ts`, `linha.ts` e os
  testes deles.
- **Critério de aceite:** spec 10, D1–D6; spec 08, C1b, C8 e C10 para o `LINE`, e a parte
  de C7 que não depende de tecla — `pnpm test command-line`.
- **Fora de escopo:** componente visual, teclas, clique na tela, outros comandos.
- **Depende de:** MT-UI-04.
- **Resultado (2026-11-02):** `pontos.ts` (`x,y`, `@dx,dy`, `d<a`, `@d<a`; ponto decimal),
  `motor.ts` (`MotorDeComandos`: prompt "Comando:", eco no histórico, resolução pela
  tabela da spec 08, texto ao comando em curso, inválida repete o prompt, recusa do
  kernel vira "O kernel recusou: <motivo>." e o comando continua, `Esc` encerra com
  "_Cancelado_", vazio repete o último iniciado; `lerOpcao` para C8) e `linha.ts` (D1–D6,
  com os prompts da versão em português do AutoCAD). A regra da camada corrente saiu da
  sonda para `camada.ts`, usada pelos dois. O `Desfazer` do LINE desfaz o passo quando a
  pilha está como o LINE a deixou e, se outra ação entrou depois, apaga a reta. O C6 já
  está no motor; a prova por tecla fica com o MT-KV-14b. Provas: `pontos.spec.ts`,
  `motor.spec.ts` e `linha.spec.ts` (46 casos, 27 vistos falhar por asserção; os 19 que
  passaram com o esboço são recusas), sobre `documento-falso.ts`.

### MT-KV-14b: Linha de comando visível, ligada ao motor (V4)

- **Objetivo:** a linha de comando própria, abaixo da tela, com prompt, histórico de
  mensagens, sugestões e as teclas `Enter`, `Espaço`, `Esc` e setas; `L` desenha pelo
  kernel e a tela reflete.
- **Arquivos no escopo:** `src/lib/components/workspace/CommandLine.svelte` (novo),
  `src/routes/+page.svelte`, `e2e/kernel-view.e2e.ts`, `e2e/command-line.e2e.ts` (novo).
- **Critério de aceite:** spec 07, V4 com a linha própria (sai o `test.fail()`); spec 08,
  C6, C7 e C12.
- **Fora de escopo:** tirar a barra do upstream (MT-KV-13b); clique na tela.
- **Depende de:** MT-KV-14a.
- **Resultado (2026-11-02):** `CommandLine.svelte`, abaixo do canvas: prompt, histórico
  (`role="log"`), sugestões (`role="listbox"`, nome em inglês, em português e apelidos)
  e as teclas `Enter`, `Espaço` (confirma, como no AutoCAD), `Esc` e setas. A página passa
  o documento do kernel e, depois de cada edição, relê o histórico do menu e a tela. O
  motor ganhou `sugerir`, que não sugere comando com outro em curso. O contrato que os
  testes observam está na spec 08. Provas: o V4 de `kernel-view.e2e.ts` desenha com `L` e
  passa sem `test.fail()`; `command-line.e2e.ts` cobre C5, C6, C7, C9 e C12 (0,8 ms do
  evento à lista). Os e2e falharam antes contra um esboço só com o contrato; o localizador
  da sugestão foi estreitado para `/^LINE LINHA/` porque `/LINE/` casava com outras
  opções. Pendências visuais: o botão flutuante "Mensagens" cobre o canto direito da
  linha, e a barra do upstream continua acima do canvas até o MT-KV-13b.

### MT-KV-14c: Seleção na tela própria e ERASE

- **Objetivo:** selecionar entidades do kernel por clique e por janela na tela própria,
  com destaque, e apagar a seleção com `ERASE`.
- **Arquivos no escopo:** a definir no detalhe, ao abrir.
- **Critério de aceite:** critérios do `ERASE` e da seleção escritos na spec 10 antes de
  implementar.
- **Fora de escopo:** grips.
- **Depende de:** MT-KV-14b.

### MT-KV-14d: Comandos de edição do K3 pela linha (V5b)

- **Objetivo:** `MOVE`, `COPY`, `ROTATE`, `MIRROR`, `SCALE`, `OFFSET`, `TRIM`, `EXTEND`,
  `FILLET` e `CHAMFER` pela linha de comando, sobre a seleção, despachados pelo kernel.
- **Arquivos no escopo:** a definir no detalhe, ao abrir.
- **Critério de aceite:** spec 07, V5b; spec 08, C10 para cada comando.
- **Fora de escopo:** comandos de anotação (fase KP).
- **Depende de:** MT-KV-14c.
- **Decidido (2026-11-02):** a recusa do V5b aparece no histórico da linha de comando,
  como no AutoCAD; a spec 07 foi ajustada.

### MT-KV-14e: Edição sobre entidade não modelada (V7)

- **Objetivo:** pedir edição sobre entidade que só o upstream desenha responde "entidade
  ainda não editável" com o tipo dela, sem mudar o documento.
- **Arquivos no escopo:** a definir no detalhe, ao abrir.
- **Critério de aceite:** spec 07, V7.
- **Fora de escopo:** editar essas entidades.
- **Depende de:** MT-KV-14c.

### MT-KV-15: Verificação na NVIDIA

- **Objetivo:** roteiro manual que abre o desenho sintético, faz pan, zoom e uma edição
  no app empacotado, em Linux e Windows, na placa NVIDIA do mantenedor. O registro diz
  que AMD e Intel são presumidas compatíveis, por o código usar só WebGL2 e canvas 2D, e
  pede relato de quem as tiver.
- **Arquivos no escopo:** `docs/verificacao-gpu.md` (novo).
- **Critério de aceite:** spec 00, N5: registro da NVIDIA nos dois sistemas, incluindo
  o WebKitGTK no Linux.
- **Fora de escopo:** correções que o roteiro revelar, e relatos futuros de AMD ou Intel
  (viram tickets).
- **Depende de:** MT-KV-12.

### MT-KV-16: Aplicar o orçamento de tamanho (ADR 0009)

- **Objetivo:** o build de release falha com o `.wasm` do kernel acima de 600 000 bytes
  ou com o total de `build/` acima do registro versionado.
- **Arquivos no escopo:** `scripts/build-kernel.mjs`, script novo de conferência do
  total em `scripts/`, registro do total (arquivo novo), `package.json`,
  `docs/specs/06-fachada-wasm.md` (W5).
- **Critério de aceite:** com o registro um byte abaixo da medida, `pnpm build` falha
  com mensagem que cita os dois números; com o registro igual, passa. O teto do kernel
  conferido do mesmo jeito.
- **Fora de escopo:** reduzir o upstream.
- **Depende de:** ADR 0009 aceito.
- **Resultado (2026-11-01):** `scripts/size-budget.json` guarda o teto (600 000) e o
  registro do total (12 921 011, folga 262 144, conforme a revisão do ADR);
  `scripts/size-budget.mjs` confere, chamado no fim do `pnpm build` e pelo build do
  kernel. Visto falhar com o registro abaixo da medida e o teto um byte abaixo dela.
