<!-- Caminho relativo: docs/CURRENT-STATE.md -->

# Estado Corrente (Handoff)

> **Fonte central de sincronização — contém apenas o estado corrente.**
> Atualizado a cada commit. Não inclua segredos.
>
> **Histórico completo:** [`docs/handoff-arquivo.md`](./handoff-arquivo.md) — rodadas
> anteriores e a tabela de hashes reescritos. Consulte sob demanda.

## Último turno

- **Data:** 2026-10-30
- **Branch:** `feat/kernel-cad-k3` · PR em rascunho
  [#2](https://github.com/psiagoleal/neocad/pull/2)
- **Commit:** o deste turno (MT-K3-12, lado TypeScript); `74169de` é a parte Rust
- **Estado da árvore:** limpa após o commit · **DoD:** `make kernel-check`, `pnpm lint`,
  `pnpm test` (98), `pnpm check` e `make e2e` (9) verdes; CI do PR a conferir após o push

## Metas cumpridas neste turno

- **Fase K3 fechada.** MT-K3-12, lado TypeScript: `CadDocument` expõe as dez operações e as
  quatro consultas com tipos NeoCAD (`CadSnap`, `CadSnapMode`); toda chamada passa por um
  guarda que converte recusa do kernel em `CadKernelOperationError` (W3), mantém o
  `CadKernelContractError` nomeando o campo (W4) e, em `WebAssembly.RuntimeError`, marca o
  documento como não confiável (`isTrusted`), de modo que toda chamada seguinte falha, menos
  `toDxf` (W4b). Costura de teste `CadDocument.fromSession`. Regra ESLint
  `no-restricted-imports` (W1b) e teto de 465 000 bytes no `scripts/build-kernel.mjs` (W5),
  provado com teto artificial. 18 testes em `cad-document-sessao.spec.ts` escritos pelo
  `qwen3-coder:30b`, 17 vistos falhando antes; correções mecânicas declaradas no commit.
  Crítica GLM-5.2: cinco achados, todos refutados. Spec 06 vigente; a linha E6 da spec 05
  ganhou a prova que estava deslocada para E6b.
- MT-K3-12, parte Rust (`74169de`): `CadSession` ganhou as dez operações da spec 05 (`moveEntities`,
  `copyEntities`, `rotateEntities`, `mirrorEntities`, `scaleEntities`, `offsetEntity`,
  `trimEntity`, `extendEntity`, `filletEntities`, `chamferEntities`), cada uma uma ação
  nomeada no histórico, e as consultas `intersections`, `entitiesInBox`, `hitTest` e `snap`.
  20 testes de host escritos em contexto limpo (subagente Sonnet, porque o gateway estava
  fora do ar), 15 vistos falhando por asserção; os 5 de recusa já passavam com o corpo
  provisório, que só recusava. Oito correções mecânicas de clippy nos testes (`clippy --fix`).
  `.wasm` de release: **414 302 bytes** (limite W5: 465 000).
- MT-K3-10, MT-K3-11 e a rodada anterior (review, ADR 0007, specs, plano) estão no
  [arquivo](handoff-arquivo.md#rodada-de-2026-10-29-k3-aparar-e-concordar).

## O que mudou de rumo

- A ordem agora é **K3 → KV → UI/UX → KP1…KP7 → K6 → K4 → K7–K9** (ADR 0007).
- Comandos digitados terão **paridade com o AutoCAD em inglês e português**, com a grafia
  oficial de cada idioma, sem tradução entre eles (spec 08 e
  [`08-comandos-autocad.tsv`](specs/08-comandos-autocad.tsv)).
- A CSP do Tauri será ligada (spec 09).
- O DXF gravado passará de `AC1021` para `AC1032` (MT-QP-06).

## Em andamento

- Nada começado.

## Próximo passo sugerido

1. Conferir o CI do PR #2 depois do push.
2. MT-KV-01: o E2E que prova a perda da linha desenhada pelo upstream (abre a fase KV).

## Impedimentos abertos

- O GLM-5.3 no gateway esgota 16–32 mil tokens raciocinando mesmo em lotes de 4 testes:
  `{"thinking":{"type":"disabled"}}` é ignorado e `chat_template_kwargs.enable_thinking=false`
  só move o raciocínio para a resposta. Para escrever testes, usar `qwen3-coder:30b`
  (~1 mil tokens por lote, formato respeitado); para crítica, `GLM-5.2-FP8 --no-think`.

- O gateway só responde com a rede da empresa ou a VPN; fora dela, a delegação cai para
  subagente (como na parte Rust do MT-K3-12).
- MT-HG-07 depende de o mantenedor instalar o `gitleaks` na máquina.
- O mecanismo de KV (espelhar no upstream ou desenhar do kernel) só se decide depois dos
  spikes MT-KV-03 e MT-KV-04; até o ADR 0008, a interface não ganha edição.

> Painel de tickets (o que existe e o que falta): [`docs/TICKETS.md`](TICKETS.md).
