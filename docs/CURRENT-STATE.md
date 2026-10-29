<!-- Caminho relativo: docs/CURRENT-STATE.md -->

# Estado Corrente (Handoff)

> **Fonte central de sincronização — contém apenas o estado corrente.**
> Atualizado a cada commit. Não inclua segredos.
>
> **Histórico completo:** [`docs/handoff-arquivo.md`](./handoff-arquivo.md) — rodadas
> anteriores e a tabela de hashes reescritos. Consulte sob demanda.

## Último turno

- **Data:** 2026-10-29
- **Branch:** `feat/kernel-cad-k3` · PR em rascunho
  [#2](https://github.com/psiagoleal/neocad/pull/2)
- **Commit:** o deste turno (MT-HG-01, 02, 04, 05 e 06)
- **Estado da árvore:** limpa · **DoD:** CI do PR verde em Linux e Windows até `45ef5d7`

## Metas cumpridas neste turno

- Review do repositório inteiro, com critérios declarados antes e crítica por modelo de
  outra família: [`docs/reviews/2026-10-29-review.md`](reviews/2026-10-29-review.md) —
  `45ef5d7`.
- Nome de desenho local retirado do repositório; o teste DWG lê `NEOCAD_REFERENCE_DWG` —
  `218b1ab`.
- Mensagens de 12 commits locais reescritas antes do primeiro envio (trailer de sessão
  proibido pelo §7); tabela de hashes no arquivo.
- [ADR 0007](adr/0007-fase-de-prancha-de-engenharia.md) aceito: fases **KV** (o kernel é o
  que se vê) e **KP** (prancha de engenharia) antes de K4 e K7–K9 — `7dda2b7`.
- Specs do projeto em [`docs/specs/`](specs/README.md), com provas conferidas e duas
  rodadas de crítica — `7dda2b7`.
- Plano em micro-tickets (HG, QP, K3, KV, UI) — `ee7d5b5`.
- MT-HG-01: ilhas do `AGENTS.md` com a stack e os comandos reais.
- MT-HG-02: este handoff, com o histórico movido para o arquivo.
- MT-HG-04: mapa de arquitetura com as arestas reais e onde a GPL está de fato.
- MT-HG-05: `.codex/` ignorado; CHANGELOG com uma linha por mudança.
- MT-HG-06: cabeçalho de caminho relativo nos 15 arquivos que faltavam.
- `make dist-test` verde: pacotes Linux e Windows (MinGW, só para teste) em `dist-test/`.

## O que mudou de rumo

- A ordem agora é **K3 → KV → UI/UX → KP1…KP7 → K6 → K4 → K7–K9** (ADR 0007).
- Comandos digitados terão **paridade com o AutoCAD em inglês e português**, com a grafia
  oficial de cada idioma, sem tradução entre eles (spec 08 e
  [`08-comandos-autocad.tsv`](specs/08-comandos-autocad.tsv)).
- A CSP do Tauri será ligada (spec 09).
- O DXF gravado passará de `AC1021` para `AC1032` (MT-QP-06).

## Em andamento

- Higiene (MT-HG-03 e MT-HG-07) e fechamento de K3 (MT-K3-10 a MT-K3-13).

## Próximo passo sugerido

1. Terminar os tickets HG.
2. MT-K3-13 (recusar fator de escala inválido), depois MT-K3-10, 11 e 12.
3. MT-KV-01: o E2E que prova a perda da linha desenhada pelo upstream.

## Impedimentos abertos

- MT-HG-07 depende de o mantenedor instalar o `gitleaks` na máquina.
- O mecanismo de KV (espelhar no upstream ou desenhar do kernel) só se decide depois dos
  spikes MT-KV-03 e MT-KV-04; até o ADR 0008, a interface não ganha edição.

> Painel de tickets (o que existe e o que falta): [`docs/TICKETS.md`](TICKETS.md).
