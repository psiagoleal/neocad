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
- **Commit:** o deste turno (MT-HG-01 a 06)
- **Estado da árvore:** com alterações não commitadas do MT-K3-10 (ver Em andamento) · **DoD:** CI do PR verde em Linux e Windows até `45ef5d7`

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
- MT-HG-03: README descreve o NeoCAD de hoje; roteiro pelos ADRs 0003 e 0007; roteiros
  antigos marcados como históricos.
- MT-HG-04: mapa de arquitetura com as arestas reais e onde a GPL está de fato.
- MT-HG-05: `.codex/` ignorado; CHANGELOG com uma linha por mudança.
- MT-HG-06: cabeçalho de caminho relativo nos 15 arquivos que faltavam.
- MT-K3-13: escala recusa fator zero, negativo ou não finito; teste escrito em contexto
  limpo a partir do critério, visto falhando antes da correção.
- `make dist-test` verde: pacotes Linux e Windows (MinGW, só para teste) em `dist-test/`.

## O que mudou de rumo

- A ordem agora é **K3 → KV → UI/UX → KP1…KP7 → K6 → K4 → K7–K9** (ADR 0007).
- Comandos digitados terão **paridade com o AutoCAD em inglês e português**, com a grafia
  oficial de cada idioma, sem tradução entre eles (spec 08 e
  [`08-comandos-autocad.tsv`](specs/08-comandos-autocad.tsv)).
- A CSP do Tauri será ligada (spec 09).
- O DXF gravado passará de `AC1021` para `AC1032` (MT-QP-06).

## Em andamento

- **MT-K3-10 (aparar e estender), começado e não commitado.** Na árvore:
  `kernel/neocad-geometry/src/trim.rs` com a API final (`trim`, `extend`, `TrimError` com
  `PickOffCurve`, `NoCuttingEdge`, `NoBoundary`, `NotExtendable`) e corpo-esboço que
  devolve a curva intacta, para os testes falharem por asserção; `lib.rs` registra o
  módulo; `sub_piece` e `polyline_from` de `offset.rs` passaram a `pub(crate)` para serem
  reaproveitados. **Falta:** escrever `kernel/neocad-geometry/tests/aparar_e_estender.rs`
  a partir dos critérios E7–E9 da spec 05 em contexto limpo e vê-lo falhar; implementar
  (trechos por parâmetro global como em `Polyline`; circunferência e polilinha fechada
  aparadas entre as duas interseções vizinhas ao ponto; extensão por raio ou pelo arco
  complementar até a primeira fronteira); depois `commands::trim`/`commands::extend`
  substituindo a entidade na mesma transação, com teste em `undo_bytes.rs`.
- MT-K3-11 e MT-K3-12 em seguida.

## Próximo passo sugerido

1. Retomar o MT-K3-10 pelo teste (ver Em andamento), depois MT-K3-11 e MT-K3-12.
2. MT-KV-01: o E2E que prova a perda da linha desenhada pelo upstream.

## Impedimentos abertos

- O GLM-5.3 no gateway esgota até 32 mil tokens raciocinando em pedidos longos, mesmo com
  `{"thinking":{"type":"disabled"}}`; pedidos curtos funcionam. Para escrever testes,
  dividir o pedido em lotes de 3–4 testes ou usar outro modelo do gateway.

- MT-HG-07 depende de o mantenedor instalar o `gitleaks` na máquina.
- O mecanismo de KV (espelhar no upstream ou desenhar do kernel) só se decide depois dos
  spikes MT-KV-03 e MT-KV-04; até o ADR 0008, a interface não ganha edição.

> Painel de tickets (o que existe e o que falta): [`docs/TICKETS.md`](TICKETS.md).
