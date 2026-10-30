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
- **Commit:** o deste turno (MT-KV-03/04 e ADR 0008 proposto); `0d1b4ce` (MT-KV-02) passou
  na CI nos 8 jobs
- **Estado da árvore:** limpa após o commit · **DoD:** `pnpm lint`, `pnpm check` e
  `make e2e` (15 passam; 2 ignorados — o DWG de referência, que é local, e a medida em
  escala, que só roda com `NEOCAD_SINTETICO_N`) verdes

## Metas cumpridas neste turno

- **Spikes de KV medidos** no desenho sintético de 100 mil entidades (protótipos nos ramos
  locais `spike/kv-espelhamento` e `spike/kv-render-kernel`, fora de `master`):
  - [espelhar no upstream](spikes/kv-espelhamento.md): abrir 8,9 s; cada edição ~2,2 s,
    gastos dentro do upstream; esvaziar o modelo dele não terminou em 13 min. Não fecha.
  - [desenhar do kernel](spikes/kv-render-kernel.md), canvas 2D com `Path2D` por camada:
    abrir ~2,8 s; edição 115–122 ms (meta 100 ms); pan 5–6 quadros/s.
  - Lacuna comum: a fachada não consulta entidade pelo id, e as operações não dizem o que
    mudaram.
- **[ADR 0008](adr/0008-tela-desenhada-a-partir-do-kernel.md) proposto:** a tela passa a
  ser desenhada do kernel; o upstream fica só com DWG e o não modelado, sem edição; meta
  nova de pan a 30 quadros/s. Crítica GLM-5.2: passa, com quatro lacunas de redação, todas
  incorporadas.
- Gerador: o espaço-papel recebia só polilinha e texto (`i % 20` alinhado com `i % 5`);
  agora conta por grupo de cinco. Teste novo visto falhando antes (`LINE em Prancha A1`,
  recebido 0).
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

- MT-KV-05: o ADR 0008 aguarda o aceite do mantenedor. Aceito, falta pôr os não-funcionais
  em EARS na spec 00 e na spec 07 e escrever os tickets do bloco B de KV.

## Próximo passo sugerido

1. Mantenedor decide o ADR 0008 (aceitar, ajustar ou recusar).
2. Aceito: specs 00 e 07 com as metas em EARS; tickets de implementação de KV (fachada
   com ids alterados e consulta por id; desenhista; retirar a barra do upstream).

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
