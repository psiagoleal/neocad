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
- **Commit:** o deste turno (MT-K3-12, parte Rust); `5728701` e este ainda **não publicados**
- **Estado da árvore:** limpa após o commit · **DoD:** `make kernel-check` verde; CI do PR a conferir após o push

## Metas cumpridas neste turno

- MT-K3-12, parte Rust: `CadSession` ganhou as dez operações da spec 05 (`moveEntities`,
  `copyEntities`, `rotateEntities`, `mirrorEntities`, `scaleEntities`, `offsetEntity`,
  `trimEntity`, `extendEntity`, `filletEntities`, `chamferEntities`), cada uma uma ação
  nomeada no histórico, e as consultas `intersections`, `entitiesInBox`, `hitTest` e `snap`.
  20 testes de host escritos em contexto limpo (subagente Sonnet, porque o gateway estava
  fora do ar), 15 vistos falhando por asserção; os 5 de recusa já passavam com o corpo
  provisório, que só recusava. Oito correções mecânicas de clippy nos testes (`clippy --fix`).
  `.wasm` de release: **414 302 bytes** (limite W5: 465 000).
- MT-K3-11: concordância (`fillet`, entre reta e arco) e chanfro (`chamfer`, entre retas),
  com `commands::fillet`/`commands::chamfer` (as duas curvas alteradas no lugar; a junção
  entra como entidade nova). Spec 05 vigente inteira (E1–E13). 18 testes escritos pelo
  `qwen3-coder:30b` e vistos falhando antes; a 1ª leva veio inutilizável e foi refeita com
  auxiliares de asserção prontos. A crítica (GLM-5.2) achou um defeito real — concordância
  que envolve o arco tocava do lado errado — reproduzido em teste e corrigido; os demais
  achados foram refutados.
- `1200f95`: tabela da spec 05 realinhada ao Prettier (o CI do frontend tinha falhado por
  isso; agora rodo `pnpm lint` sempre que mexo em docs).
- MT-K3-10: aparar e estender (`trim`, `extend`) para reta, arco, circunferência e
  polilinha, e os comandos `commands::trim`/`commands::extend` no editor transacional
  (um passo de desfazer cada; o primeiro pedaço mantém o identificador da entidade).
  Spec 05 vigente até E9. Testes escritos em contexto limpo pelo `qwen3-coder:30b` do
  gateway, vistos falhando por asserção (16 de geometria, 5 de comando) antes da
  implementação; duas correções mecânicas declaradas (nome de variável que escondia o
  auxiliar; bytes de "antes" capturados depois da edição). Crítica por GLM-5.2 e qwen:
  todos os achados conferidos e refutados.
- A rodada anterior (review, ADR 0007, specs, plano, HG-01..06, MT-K3-13) foi para o
  [arquivo](handoff-arquivo.md#rodada-de-2026-10-29-review-e-reorganização).

## O que mudou de rumo

- A ordem agora é **K3 → KV → UI/UX → KP1…KP7 → K6 → K4 → K7–K9** (ADR 0007).
- Comandos digitados terão **paridade com o AutoCAD em inglês e português**, com a grafia
  oficial de cada idioma, sem tradução entre eles (spec 08 e
  [`08-comandos-autocad.tsv`](specs/08-comandos-autocad.tsv)).
- A CSP do Tauri será ligada (spec 09).
- O DXF gravado passará de `AC1021` para `AC1032` (MT-QP-06).

## Em andamento

- MT-K3-12: falta o lado TypeScript — tipos em `src/lib/types/cad.ts`, métodos no
  `cad-document.ts` com erros tipados (W3), conversão validada (W4), envenenamento após
  pânico/trap (W4b, com sessão simulada; o construtor de `CadDocument` precisa de uma
  costura de teste), regra `no-restricted-imports` (W1b) e checagem de tamanho no
  `scripts/build-kernel.mjs` (W5). Então `pnpm lint`, `pnpm test`, `pnpm check`, `make e2e`.

## Próximo passo sugerido

1. Publicar `5728701` e o commit do MT-K3-12: a conta ativa do `gh` é a de trabalho; o
   mantenedor troca com `gh auth switch -u psiagoleal`.
2. Terminar o MT-K3-12 (lado TypeScript), que fecha a fase K3.
3. MT-KV-01: o E2E que prova a perda da linha desenhada pelo upstream.

## Impedimentos abertos

- O GLM-5.3 no gateway esgota 16–32 mil tokens raciocinando mesmo em lotes de 4 testes:
  `{"thinking":{"type":"disabled"}}` é ignorado e `chat_template_kwargs.enable_thinking=false`
  só move o raciocínio para a resposta. Para escrever testes, usar `qwen3-coder:30b`
  (~1 mil tokens por lote, formato respeitado); para crítica, `GLM-5.2-FP8 --no-think`.

- Gateway (10.0.35.2:4000) inacessível em 2026-10-29 à tarde (conexão expirou): sem VPN
  ou rede da empresa, a delegação cai para subagente.
- MT-HG-07 depende de o mantenedor instalar o `gitleaks` na máquina.
- O mecanismo de KV (espelhar no upstream ou desenhar do kernel) só se decide depois dos
  spikes MT-KV-03 e MT-KV-04; até o ADR 0008, a interface não ganha edição.

> Painel de tickets (o que existe e o que falta): [`docs/TICKETS.md`](TICKETS.md).
