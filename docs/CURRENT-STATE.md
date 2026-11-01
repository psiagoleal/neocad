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
- **Commit:** `607604c` (MT-KV-11); antes, `889be19` (MT-KV-12)
- **Estado da árvore:** limpa após o commit · **DoD:** ver a mensagem do commit
  (`make kernel-check`, `pnpm build` com o orçamento, `pnpm lint`, `pnpm check`,
  `pnpm test`)

## Metas cumpridas neste turno

- **MT-KV-11:** o texto do kernel aparece na tela, nos dois caminhos de desenho. A fonte
  é a Hershey Roman Simplex, embutida em `neocad-geometry`
  ([ADR 0010](adr/0010-texto-da-tela-em-fonte-de-traco.md)). O mantenedor escolheu a
  fonte e autorizou o download. O kernel compõe os acentos e entende `%%d`, `%%p`,
  `%%c`, `%%%` e `%%nnn`. O texto vira segmentos no bloco da camada, então o desenhista
  em TypeScript não mudou.
- **V1 com texto** provado em `e2e/kernel-view.e2e.ts` (`with-text.dxf`), visto falhar
  com o texto desligado. Uma amostra com acentos e texto girado foi conferida em
  captura.
- Com Chrome em janela na RTX 3070 e 100 mil entidades, a espera de N1 voltou a ser
  `contagem.modelo`: N1 2,79 / 2,89 / 2,92 s, N2 12,9–15,4 ms, N3 16,7 ms. O `.wasm`
  passou a 479 595 bytes (+14 872; teto 600 000).
- A rodada anterior (MT-KV-12) está no
  [arquivo](handoff-arquivo.md#rodada-de-2026-11-01-a-tela-acompanha-a-edição).

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

1. MT-KV-13: o upstream só desenha o que o kernel não modela.
2. MT-KV-14: comandos de edição digitados chegam ao kernel (fecha o V4).
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
- O texto sai sempre na fonte Hershey: estilo, fator de largura, inclinação, alinhamento
  e `%%u`/`%%o` ficam para a fase KP (ADR 0010).
- O erro de corda é um só para todos os espaços (lado do modelo / 4096); num desenho
  cujo modelo é muito maior que a prancha, os arcos do layout saem poligonais.
- MT-HG-07 depende de o mantenedor instalar o `gitleaks` na máquina.

> Painel de tickets (o que existe e o que falta): [`docs/TICKETS.md`](TICKETS.md).
