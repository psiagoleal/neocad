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
- **Commit:** `e79e1f8` (MT-KV-14d4); antes, `7052035` (MT-KV-14d3)
- **Estado da árvore:** limpa após o commit · **DoD:** ver a mensagem do commit
  (`make kernel-check`, `pnpm build` com o orçamento, `pnpm lint`, `pnpm check`,
  `pnpm test`)

## Metas cumpridas neste turno

- **MT-KV-14c fatiado** (commit `75a9f37`): 14c é a seleção; o `ERASE` foi para o 14f.
  Critérios S1–S6 e E1–E3 na spec 10.
- **MT-KV-14c:** clique, janela e cruzada na tela própria, com destaque, consultando o
  kernel por espaço (`pickOnScreen`, `selectOnScreen`, `getScreenSegmentsOf`).
  `data-selecao` exposto. O índice espacial passou a ser feito ao preparar a tela e
  atualizado aos poucos depois das edições (clique: 197 → 4–10 ms com 100 mil).
- Corrigido: `getEntitiesById` recusava entidade apagada (`undefined` da fachada).
- Lacuna registrada: janela em volta do desenho inteiro (~90 mil) leva ~290 ms; fica
  `test.fail()` até o MT-KV-14g (seleção mantida no kernel).
- **MT-KV-14f:** `ERASE` / `APAGAR` / `E` apaga a seleção num só passo de desfazer, com
  prompt "Selecione objetos:" quando não há seleção; `Delete` fora de campo faz o mesmo.
  A fachada ganhou `removeEntities` (tudo ou nada).
- **MT-KV-14d fatiado** em 14d1–14d5 (commit `b431fc8`); **MT-KV-14d1:** `MOVE` e
  `COPY` sobre a seleção, com a etapa "Selecione objetos:" comum aos comandos de edição.
- **MT-KV-14d2:** `ROTATE`, `MIRROR` (mantém as originais por padrão, num só passo, pelo
  novo `mirrorCopyEntities`) e `SCALE` pela linha.
- **MT-KV-14d3:** `OFFSET` pela linha, com o objeto escolhido pelo clique; o motor passou
  a receber da tela o aviso de seleção. Aberto o MT-KV-14h (pontos pelo clique na tela).
- **MT-KV-14d4:** `TRIM` e `EXTEND`; a tela ganhou o modo de escolha (o clique vai ao
  comando com o ponto e não seleciona), que também passou a servir ao `OFFSET`.
- A rodada anterior (MT-KV-13b e MT-UI-05) está no
  [arquivo](handoff-arquivo.md#rodada-de-2026-11-02-upstream-sem-comandos).

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

1. MT-KV-14d5 (`FILLET`, `CHAMFER`), com critérios na spec 10 antes; o modo de escolha
   já serve aos dois cliques. Depois 14h (pontos pelo clique, que faz falta ao LINE).
   Depois 14e (V7) e 14g (seleção no kernel).
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
