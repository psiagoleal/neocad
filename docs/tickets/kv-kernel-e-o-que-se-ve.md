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

### MT-KV-10: Desenhista pela GPU com reserva em canvas 2D

- **Objetivo:** desenhar retas, arcos, círculos e polilinhas do kernel, nos dois espaços,
  com pan e zoom pela matriz da vista; cair sozinho no canvas 2D sem WebGL2.
- **Arquivos no escopo:** `src/lib/render/` (novo), `src/lib/viewer/neocad-viewer.ts`,
  `e2e/kernel-document.e2e.ts`, `e2e/kernel-view.e2e.ts`.
- **Critério de aceite:** spec 07, V1 (sem texto); spec 00, N1, N3 e N4, com o
  MT-KV-07 passando localmente e o resultado citado no PR.
- **Fora de escopo:** texto; edição.
- **Depende de:** MT-KV-07, MT-KV-09.

### MT-KV-11: Texto do kernel na tela

- **Objetivo:** posição dos glifos calculada no kernel, desenho pelo caminho da GPU e
  pelo de reserva.
- **Arquivos no escopo:** a definir no MT-KV-09, conforme a camada escolhida.
- **Critério de aceite:** spec 07, V1 completo; N1 e N3 mantidos.
- **Fora de escopo:** MTEXT e estilos de texto (fase KP).
- **Depende de:** MT-KV-10.

### MT-KV-12: Refletir edição, desfazer e refazer

- **Objetivo:** a tela atualiza só o que os ids da operação indicam.
- **Arquivos no escopo:** `src/lib/render/`, `e2e/kernel-view.e2e.ts`.
- **Critério de aceite:** spec 07, V2 e V3; spec 00, N2, pelo MT-KV-07.
- **Fora de escopo:** comandos digitados.
- **Depende de:** MT-KV-10.

### MT-KV-13: Upstream só para o que o kernel não modela

- **Objetivo:** o upstream deixa de desenhar entidade modelada, desenha o resto numa
  camada abaixo, na mesma vista; a barra de comandos e a edição do upstream saem.
- **Arquivos no escopo:** `src/lib/viewer/neocad-viewer.ts`, `src/routes/+page.svelte`,
  `src/lib/services/cad-commands.ts`.
- **Critério de aceite:** spec 07, V5 e V6.
- **Fora de escopo:** linha de comando própria (spec 08).
- **Depende de:** MT-KV-12.

### MT-KV-14: Comandos de edição pelo kernel na linha de comando própria

- **Objetivo:** LINE e os comandos de edição de K3 digitados chegam ao kernel; recusa do
  kernel e entidade não modelada viram mensagem.
- **Arquivos no escopo:** `src/lib/commands/`, `e2e/kernel-view.e2e.ts`.
- **Critério de aceite:** spec 07, V4 (sai o `test.fail()`), V5b e V7.
- **Fora de escopo:** comandos de anotação (fase KP).
- **Depende de:** MT-KV-13, MT-UI-04.

### MT-KV-15: Verificação em AMD, NVIDIA e Intel

- **Objetivo:** roteiro manual que abre o desenho sintético, faz pan, zoom e uma edição
  no app empacotado, em Linux e Windows, com resultado registrado por fabricante.
- **Arquivos no escopo:** `docs/verificacao-gpu.md` (novo).
- **Critério de aceite:** spec 00, N5: registro das três marcas nos dois sistemas,
  incluindo a NVIDIA com WebKitGTK.
- **Fora de escopo:** correções que o roteiro revelar (viram tickets).
- **Depende de:** MT-KV-12; máquinas AMD e Intel além da do mantenedor.
