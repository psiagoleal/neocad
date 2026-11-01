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
- **Commit:** o deste turno (MT-KV-10, em andamento); antes, `8610d47` (MT-KV-16)
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
- **MT-KV-10 (em andamento):** a tela própria desenha o DXF lido pelo kernel.
  `src/lib/render/` traz a vista (pan, zoom no cursor, enquadrar), a cor da camada
  (paleta de índice calculada), o desenhista WebGL2, a reserva em canvas 2D e o
  controlador; `KernelScreen.svelte` põe o canvas e as abas dos espaços sobre a área do
  upstream. O kernel ganhou `screenExtents` e `screenEntityCount` (`.wasm` 464 723
  bytes). Provados: V1 sem texto no modelo e em cada layout, N4 (reserva) e N1 (2,7 s
  com 100 mil entidades).
- **N3 não pôde ser medido na GPU.** Causa: o driver da NVIDIA foi atualizado (biblioteca
  595.99) sem reiniciar a máquina; `nvidia-smi` responde "Driver/library version
  mismatch". Sem a GPU, o Chrome usa WebGL2 em software (SwiftShader), e o quadro custa
  ~180 ms; com `--ignore-gpu-blocklist`, o processo de GPU cai três vezes e o WebGL some.
  Perfil de CPU confirmou: o tempo é nativo do navegador, não do código da tela. O
  teste de N3 passou a recusar renderizador em software.
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

- MT-KV-10: falta medir N3 na GPU, depois de reiniciar a máquina.

## Próximo passo sugerido

1. Mantenedor: reiniciar a máquina (driver NVIDIA). Então
   `NEOCAD_SINTETICO_N=100000 npx playwright test desempenho -g "N1|N3" --headed` e,
   com N3 verde, fechar o MT-KV-10.
2. MT-KV-12 (sonda, N2, refletir edição) ou MT-KV-11 (texto), que não dependem da GPU.
3. Em paralelo, quando couber: MT-UI-01 (CSP), que não depende de KV.

## Impedimentos abertos

- O GLM-5.3 no gateway esgota 16–32 mil tokens raciocinando mesmo em lotes de 4 testes:
  `{"thinking":{"type":"disabled"}}` é ignorado e `chat_template_kwargs.enable_thinking=false`
  só move o raciocínio para a resposta. Para escrever testes, usar `qwen3-coder:30b`
  (~1 mil tokens por lote, formato respeitado); para crítica, `GLM-5.2-FP8 --no-think`.

- O gateway só responde com a rede da empresa ou a VPN; fora dela, a delegação cai para
  subagente (como na parte Rust do MT-K3-12).
- **GPU desta máquina indisponível até reiniciar** (driver NVIDIA atualizado sem
  reinício). Bloqueia só a medida de N3.
- A reserva em canvas 2D vale para a tela própria; num navegador sem WebGL2 o upstream
  não monta e a abertura de arquivo fica desabilitada, até o MT-KV-13.
- O erro de corda é um só para todos os espaços (lado do modelo / 4096); num desenho
  cujo modelo é muito maior que a prancha, os arcos do layout saem poligonais.
- MT-HG-07 depende de o mantenedor instalar o `gitleaks` na máquina.

> Painel de tickets (o que existe e o que falta): [`docs/TICKETS.md`](TICKETS.md).
