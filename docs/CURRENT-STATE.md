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
- **Commit:** o deste turno (MT-KV-02); `3e2e3e4` (MT-KV-01) passou na CI nos 8 jobs
- **Estado da árvore:** limpa após o commit · **DoD:** `pnpm lint`, `pnpm check` e
  `make e2e` (14 passam; 2 ignorados — o DWG de referência, que é local, e a medida em
  escala, que só roda com `NEOCAD_SINTETICO_N`) verdes

## Metas cumpridas neste turno

- MT-KV-02: `scripts/gerar-dxf-sintetico.mjs` gera, a partir de uma semente, um DXF
  AC1015 com LINE, CIRCLE, ARC, LWPOLYLINE e TEXT em L camadas, 90% no espaço-modelo e 5%
  em cada um de dois layouts, e imprime a contagem esperada
  (`node scripts/gerar-dxf-sintetico.mjs --entidades 100000 --camadas 20 --saida x.dxf`).
  `e2e/dxf-sintetico.e2e.ts` confere contagem, texto e determinismo, e que o kernel abre
  2 000 entidades com a contagem exata e grava sem perda. **Medida:** com
  `NEOCAD_SINTETICO_N=100000`, a mensagem do kernel aparece **2,4 s** após escolher o
  arquivo (meta da spec 07: abrir em até 5 s). O desenho pelo upstream não foi medido; é
  trabalho dos spikes. Teste do `qwen3-coder:30b`, visto falhando no esboço vazio; uma
  premissa errada do pedido (o espaço-papel como perda ao salvar, que deixou de ser no
  MT-KL-09) foi trocada por "grava sem perda". Crítica GLM-5.2: 5 achados; 4 refutados
  (`$HANDSEED` confere, handles de `BLOCK` e de `BLOCK_RECORD` são distintos por norma,
  layouts usam `*Paper_Space`/`*Paper_Space0`); 1 corrigido (semente inválida virava 0).
- **Fase KV aberta.** MT-KV-01: `e2e/kernel-view.e2e.ts` desenha uma linha de (0,0) a
  (100,50) pela barra do upstream, salva, conta as `LINE` do DXF baixado e reabre. Sem o
  `test.fail()`, o teste falha na contagem, depois de todos os prompts do comando passarem:

  ```
  Expected: 2
  Received: 1
  > 92 | 	expect(contarLinhas(await readFile(destino, 'utf-8'))).toBe(2);
  ```

  A linha aparece na tela e não vai para o arquivo: a barra edita o modelo do upstream, e
  salvar grava o do kernel (achado C4). Na barra do upstream (`.ml-cli-text`) só entram nomes
  de comando e palavras-chave; os pontos entram nas duas caixas flutuantes junto ao cursor.
  O teste foi escrito pelo `qwen3-coder:30b`; quatro correções mecânicas (esperas que não
  esperavam, `await` faltando, `setTimeout` fora do teste, prompts não conferidos) estão
  declaradas no commit.

- K3 (MT-K3-10 a 12) está no
  [arquivo](handoff-arquivo.md#rodada-de-2026-10-30-k3-fechada).

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

1. MT-KV-03 e MT-KV-04: os dois spikes, sobre o desenho de 100 mil entidades do gerador.
2. MT-KV-05: o ADR 0008, com base nos dois relatórios.

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
