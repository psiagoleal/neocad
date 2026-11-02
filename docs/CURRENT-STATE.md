<!-- Caminho relativo: docs/CURRENT-STATE.md -->

# Estado Corrente (Handoff)

> **Fonte central de sincronização — contém apenas o estado corrente.**
> Atualizado a cada commit. Não inclua segredos.
>
> **Histórico completo:** [`docs/handoff-arquivo.md`](./handoff-arquivo.md) — rodadas
> anteriores e a tabela de hashes reescritos. Consulte sob demanda.

## Último turno

- **Data:** 2026-11-02
- **Branch:** `feat/kernel-cad-k3` · PR em rascunho
  [#2](https://github.com/psiagoleal/neocad/pull/2)
- **Commit:** `8d868f6` (MT-KV-13b); antes, `8ac0a83` (MT-KV-14b)
- **Estado da árvore:** limpa após o commit · **DoD:** ver a mensagem do commit
  (`make kernel-check`, `pnpm build` com o orçamento, `pnpm lint`, `pnpm check`,
  `pnpm test`)

## Metas cumpridas neste turno

- **Decisões do mantenedor:** a recusa do V5b fica no histórico da linha de comando; o
  modo de vírgula decimal vira o MT-UI-08 (commit `4946991`); o ADR 0011 substitui o item
  1 do ADR 0001 (catálogo).
- **MT-KV-13b + MT-UI-05:** o upstream não oferece mais comando algum (barra escondida,
  `sendStringToExecute` inerte). O catálogo `Ajuda > Comandos CAD` vem da tabela da spec
  08 e do registro `src/lib/command-line/comandos.ts`, bilíngue, com disponibilidade.
- Corrigida uma regressão do MT-KV-13a vista nas capturas: a cor 7 do upstream saía
  preta no tema escuro.
- A rodada anterior (MT-KV-14b) está no
  [arquivo](handoff-arquivo.md#rodada-de-2026-11-02-linha-de-comando-visível).

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

1. MT-KV-14c: seleção na tela própria e `ERASE` (escrever antes os critérios na spec
   10). Depois 14d (edição do K3 pela linha; V5b no histórico) e 14e (V7).
2. MT-UI-08 (vírgula decimal) quando couber.
3. Em paralelo, quando couber: MT-UI-01 (CSP), que não depende de KV.

## Impedimentos abertos

- O GLM-5.3 no gateway esgota 16–32 mil tokens raciocinando mesmo em lotes de 4 testes:
  `{"thinking":{"type":"disabled"}}` é ignorado e `chat_template_kwargs.enable_thinking=false`
  só move o raciocínio para a resposta. Para escrever testes, usar `qwen3-coder:30b`
  (~1 mil tokens por lote, formato respeitado); para crítica, `GLM-5.2-FP8 --no-think`.

- O gateway só responde com a rede da empresa ou a VPN; fora dela, a delegação cai para
  subagente (como na parte Rust do MT-K3-12).
- A reserva em canvas 2D vale para a tela própria; num navegador sem WebGL2 o upstream
  não monta e a abertura de arquivo fica desabilitada: o upstream ainda lê DWG e desenha
  o não modelado, e sai de vez só com K5/K6. N1 e N2 no
  canvas 2D (spec 00, N4) ainda não foram medidos com 100 mil entidades.
- O texto sai sempre na fonte Hershey: estilo, fator de largura, inclinação, alinhamento
  e `%%u`/`%%o` ficam para a fase KP (ADR 0010).
- O erro de corda é um só para todos os espaços (lado do modelo / 4096); num desenho
  cujo modelo é muito maior que a prancha, os arcos do layout saem poligonais.
- Um `INSERT` segue inteiro com o upstream, inclusive as linhas do bloco, porque o kernel
  não modela referência de bloco; num layout, o upstream fica escondido sob a tela opaca.
- No pnpm 11, `pnpm test -- X` roda a suíte inteira (o `--` chega ao Vitest); o filtro é
  `pnpm test X`. As provas das specs escritas com `--` continuam valendo, só não filtram.
- O botão flutuante "Mensagens" cobre o canto direito da linha de comando: ajuste da
  fase de UI.
- MT-HG-07 depende de o mantenedor instalar o `gitleaks` na máquina.

> Painel de tickets (o que existe e o que falta): [`docs/TICKETS.md`](TICKETS.md).
