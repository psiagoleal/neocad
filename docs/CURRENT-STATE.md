<!-- Caminho relativo: docs/CURRENT-STATE.md -->

# Estado Corrente (Handoff)

> **Fonte central de sincronização — contém apenas o estado corrente.**
> Atualizado a cada commit. Não inclua segredos.
>
> **Histórico completo:** [`docs/handoff-arquivo.md`](./handoff-arquivo.md) — rodadas
> anteriores e a tabela de hashes reescritos. Consulte sob demanda.

## Último turno

- **Data:** 2026-11-01
- **Branch:** `feat/kernel-cad-k3` · PR em rascunho
  [#2](https://github.com/psiagoleal/neocad/pull/2)
- **Commit:** o deste turno (MT-KV-16); antes, `9181f3c` (MT-KV-09, CI verde nos 8 jobs)
- **Estado da árvore:** limpa após o commit · **DoD:** ver a mensagem do commit
  (`make kernel-check`, `pnpm build` com o orçamento, `pnpm lint`, `pnpm check`,
  `pnpm test`)

## Metas cumpridas neste turno

- **[ADR 0009](adr/0009-orcamento-de-tamanho-do-aplicativo.md) aceito**, com uma revisão
  antes de implementar: a catraca do total ganhou folga fixa de 256 KiB. Sem ela, o
  build variava dezenas de bytes entre execuções iguais (carimbo de versão do SvelteKit)
  e cada ticket de interface exigiria ADR; a folga escolhida foi do mantenedor.
- **MT-KV-16:** `scripts/size-budget.json` (teto 600 000; registro 12 921 011 + folga
  262 144) e `scripts/size-budget.mjs`, chamado no fim do `pnpm build` (`pnpm
size:check`) e pelo build do kernel. Testes em `scripts/size-budget.spec.mjs` (o
  Vitest agora inclui `scripts/**/*.spec.mjs`); visto falhar no build real com o
  registro abaixo da medida e com o teto um byte abaixo do `.wasm`.
- A rodada anterior (MT-KV-09, ADR 0008) está no
  [arquivo](handoff-arquivo.md#rodada-de-2026-10-31-camada-da-gpu).

## O que mudou de rumo

- A ordem agora é **K3 → KV → UI/UX → KP1…KP7 → K6 → K4 → K7–K9** (ADR 0007).
- Comandos digitados terão **paridade com o AutoCAD em inglês e português**, com a grafia
  oficial de cada idioma, sem tradução entre eles (spec 08 e
  [`08-comandos-autocad.tsv`](specs/08-comandos-autocad.tsv)).
- A CSP do Tauri será ligada (spec 09).
- O DXF gravado passará de `AC1021` para `AC1032` (MT-QP-06).

## Em andamento

- Nada em andamento; o bloco B de KV segue pelo MT-KV-10.

## Próximo passo sugerido

1. MT-KV-10: desenhista WebGL2 com reserva em canvas 2D (gasta um pouco da folga do
   total; não mexe no `.wasm`).
2. Em paralelo, quando couber: MT-UI-01 (CSP), que não depende de KV.

## Impedimentos abertos

- O GLM-5.3 no gateway esgota 16–32 mil tokens raciocinando mesmo em lotes de 4 testes:
  `{"thinking":{"type":"disabled"}}` é ignorado e `chat_template_kwargs.enable_thinking=false`
  só move o raciocínio para a resposta. Para escrever testes, usar `qwen3-coder:30b`
  (~1 mil tokens por lote, formato respeitado); para crítica, `GLM-5.2-FP8 --no-think`.

- O gateway só responde com a rede da empresa ou a VPN; fora dela, a delegação cai para
  subagente (como na parte Rust do MT-K3-12).
- MT-HG-07 depende de o mantenedor instalar o `gitleaks` na máquina.

> Painel de tickets (o que existe e o que falta): [`docs/TICKETS.md`](TICKETS.md).
