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

Escrito depois do MT-KV-05.
