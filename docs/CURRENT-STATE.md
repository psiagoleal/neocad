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
- **Commit:** o deste turno (MT-KV-12); antes, `9ad5f27` (MT-KV-10 fechado)
- **Estado da árvore:** limpa após o commit · **DoD:** ver a mensagem do commit
  (`make kernel-check`, `pnpm build` com o orçamento, `pnpm lint`, `pnpm check`,
  `pnpm test`)

## Metas cumpridas neste turno

- **MT-KV-12:** a tela acompanha cada edição do kernel e o desfazer/refazer do menu
  `Editar`, buscando de novo só os blocos que a operação mudou. O canvas expõe também
  `data-caixa` (a caixa envolvente do espaço). V2 (criar, mover e apagar uma linha) e V3
  provados em `e2e/kernel-view.e2e.ts`.
- **Sonda** (`src/lib/services/sonda.ts`): `desenharLinha`, `mover` e `apagar`, na
  camada `0`. O build dos testes E2E sempre a traz (`playwright.config.ts`); o de
  produção não (conferido: nenhum arquivo de `build/` a contém).
- **N2** com Chrome em janela na RTX 3070, 100 mil entidades, três rodadas: 11,3 / 14,3 /
  11,2 ms (meta 100 ms). Na mesma rodada, N1 2,70–2,73 s e N3 16,7 ms.
- A rodada anterior (ADR 0009, MT-KV-16, MT-KV-10) está no
  [arquivo](handoff-arquivo.md#rodada-de-2026-11-01-orçamento-e-tela-própria).

## O que mudou de rumo

- A ordem agora é **K3 → KV → UI/UX → KP1…KP7 → K6 → K4 → K7–K9** (ADR 0007).
- Comandos digitados terão **paridade com o AutoCAD em inglês e português**, com a grafia
  oficial de cada idioma, sem tradução entre eles (spec 08 e
  [`08-comandos-autocad.tsv`](specs/08-comandos-autocad.tsv)).
- A CSP do Tauri será ligada (spec 09).
- O DXF gravado passará de `AC1021` para `AC1032` (MT-QP-06).

## Em andamento

- Nada em andamento.

## Próximo passo sugerido

1. MT-KV-11: texto do kernel na tela; a espera do N1 volta a ser `contagem.modelo`.
2. MT-KV-13 a 15.
3. Em paralelo, quando couber: MT-UI-01 (CSP), que não depende de KV.

## Impedimentos abertos

- O GLM-5.3 no gateway esgota 16–32 mil tokens raciocinando mesmo em lotes de 4 testes:
  `{"thinking":{"type":"disabled"}}` é ignorado e `chat_template_kwargs.enable_thinking=false`
  só move o raciocínio para a resposta. Para escrever testes, usar `qwen3-coder:30b`
  (~1 mil tokens por lote, formato respeitado); para crítica, `GLM-5.2-FP8 --no-think`.

- O gateway só responde com a rede da empresa ou a VPN; fora dela, a delegação cai para
  subagente (como na parte Rust do MT-K3-12).
- A reserva em canvas 2D vale para a tela própria; num navegador sem WebGL2 o upstream
  não monta e a abertura de arquivo fica desabilitada, até o MT-KV-13. N1 e N2 no
  canvas 2D (spec 00, N4) ainda não foram medidos com 100 mil entidades.
- O erro de corda é um só para todos os espaços (lado do modelo / 4096); num desenho
  cujo modelo é muito maior que a prancha, os arcos do layout saem poligonais.
- MT-HG-07 depende de o mantenedor instalar o `gitleaks` na máquina.

> Painel de tickets (o que existe e o que falta): [`docs/TICKETS.md`](TICKETS.md).
