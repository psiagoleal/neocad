<!-- Caminho relativo: docs/CURRENT-STATE.md -->

# Estado Corrente (Handoff)

> **Fonte central de sincronização — contém apenas o estado corrente.**
> Atualizado a cada commit. Não inclua segredos.
>
> **Histórico completo:** [`docs/handoff-arquivo.md`](./handoff-arquivo.md) — rodadas
> anteriores e a tabela de hashes reescritos. Consulte sob demanda.

## Último turno

- **Data:** 2026-11-03
- **Branch:** `feat/kernel-cad-k3` · PR em rascunho
  [#2](https://github.com/psiagoleal/neocad/pull/2)
- **Commit:** ver `git log` (MT-KV-14g, parte 2); antes, `9dd4617` (parte 1)
- **Estado da árvore:** limpa após o commit · **DoD:** ver a mensagem do commit
  (`make kernel-check`, `pnpm build` com o orçamento, `pnpm lint`, `pnpm check`,
  `pnpm test`)

## Metas cumpridas neste turno

- **MT-KV-14g:** a seleção mora no kernel (`select_area_into`, `change_selection`,
  `selection`, `selection_count`, poda a cada edição); a tela guarda só a janela aberta e
  a contagem. O destaque não copia segmentos: o kernel dá as faixas (primeiro vértice e
  contagem) dos blocos que a placa já tem, redesenhadas com `WEBGL_multi_draw` (ou na CPU,
  no canvas 2D). A área aceita direto a entidade com caixa dentro, e o índice espacial
  deixou de ordenar achados grandes. **S6 com 90 001 selecionadas: ~350 → 69 ms**, sem
  `test.fail()`; clique 6–11 ms. Mudanças de comportamento: a seleção vive num espaço só
  e os ids vêm em ordem de id. Corrigido: trocar o tema apagava o destaque com a seleção
  viva. `.wasm` 523 992 bytes (+11 368). Parte 1 em `9dd4617`; parte 2 no commit acima.
- Crítica do GLM-5.2 ainda não rodou: o gateway não resolvia (fora da rede da empresa).
- A rodada anterior (MT-KV-14c a 14e, 14h) está no
  [arquivo](handoff-arquivo.md#rodada-de-2026-11-02-seleção-e-edição-na-tela).

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

1. Rodar a crítica do GLM-5.2 sobre o MT-KV-14g quando o gateway responder (pedido: diff
   de `660223b` até o commit da parte 2).
2. MT-UI-08 (vírgula decimal) ou MT-KV-15; depois a fase de UI (aparência do marcador
   por modo de prendimento e linha elástica até o cursor).
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
