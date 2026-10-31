<!-- Caminho relativo: docs/CURRENT-STATE.md -->

# Estado Corrente (Handoff)

> **Fonte central de sincronização — contém apenas o estado corrente.**
> Atualizado a cada commit. Não inclua segredos.
>
> **Histórico completo:** [`docs/handoff-arquivo.md`](./handoff-arquivo.md) — rodadas
> anteriores e a tabela de hashes reescritos. Consulte sob demanda.

## Último turno

- **Data:** 2026-10-31
- **Branch:** `feat/kernel-cad-k3` · PR em rascunho
  [#2](https://github.com/psiagoleal/neocad/pull/2)
- **Commit:** o deste turno (MT-KV-09, só documentação); antes, `fe35d0c` (MT-KV-08, CI verde nos 8 jobs)
- **Estado da árvore:** limpa após o commit · **DoD:** `make kernel-check`, `make
kernel-wasm` (463 233 bytes, teto 465 000), `pnpm lint`, `pnpm check`, `pnpm test` (117)
  e `make e2e` (15 passam; 5 pulados de propósito) verdes

## Metas cumpridas neste turno

- **MT-KV-09 (spike):** a camada da GPU é **WebGL2 em TypeScript**
  ([relatório](spikes/kv-camada-gpu.md)). Medido com janela na RTX 3070, 100 mil
  entidades: abrir 2,2 s (N1 ≤ 5 s; ~2 s são a leitura do DXF), uma linha nova até o
  quadro 7,5 ms (N2 ≤ 100 ms), pan e zoom no limite do monitor, 16,7 ms (N3 ≤ 33 ms).
  `wgpu` empata no desempenho e leva o `.wasm` de 463 233 a 2 042 252 bytes. Protótipo
  no ramo local `spike/kv-camada-gpu` (`d2413f7`), fora de `master`.
- **[ADR 0009](adr/0009-orcamento-de-tamanho-do-aplicativo.md) proposto:** teto do
  `.wasm` do kernel de 465 000 para 600 000 bytes, e catraca no total que o aplicativo
  baixa (hoje 12 921 011 bytes, 95% do upstream), que só desce quando o upstream sai.
  Aguarda o mantenedor; o MT-KV-16 o aplica.
- **MT-KV-08** (`fe35d0c`), **MT-KV-07** (`878a7da`) e **MT-KV-06** estão no
  [arquivo](handoff-arquivo.md#rodada-de-2026-10-30-kv-prova-e-medida).
- **[ADR 0008](adr/0008-tela-desenhada-a-partir-do-kernel.md) aceito pelo mantenedor**,
  com dois acréscimos pedidos por ele:
  - **GPU:** desenho por WebGL2, independente de fabricante (AMD, NVIDIA, Intel) pela
    camada do navegador, com reserva automática em canvas 2D; WebGPU fica para depois.
  - **Divisão de responsabilidade:** o kernel prepara a geometria para a tela (segmentos,
    texto, índice espacial, seleção) em blocos `Float32Array`; o TypeScript só os repassa
    à GPU com a matriz da vista. A camada da GPU (WebGL2 direto ou `wgpu` em Rust) se
    escolhe por medida no MT-KV-09; three.js descartado.
- Spec 00 com os não-funcionais N1–N5 em EARS (abrir ≤ 5 s, editar ≤ 100 ms, pan ≥ 30
  quadros/s, reserva 2D, APIs independentes de fabricante); spec 07 aponta para eles.
  Por decisão do mantenedor, a verificação é só na NVIDIA dele; AMD e Intel ficam
  presumidas compatíveis até haver relato em contrário.
- Bloco B de KV escrito: MT-KV-06 a 15. Crítica GLM-5.2 (duas execuções): uma correção
  incorporada (no Linux o WebGL do WebKitGTK passa pelo ANGLE); as demais refutadas — o
  MT-KV-07 falhar antes do desenhista é o teste-primeiro de propósito, as dependências
  citadas chegam por transitividade, e `wgpu` tem caminho WebGPU no navegador.
- KV, bloco A (MT-KV-01 a 04), está no
  [arquivo](handoff-arquivo.md#rodada-de-2026-10-30-kv-prova-e-medida).

## O que mudou de rumo

- A ordem agora é **K3 → KV → UI/UX → KP1…KP7 → K6 → K4 → K7–K9** (ADR 0007).
- Comandos digitados terão **paridade com o AutoCAD em inglês e português**, com a grafia
  oficial de cada idioma, sem tradução entre eles (spec 08 e
  [`08-comandos-autocad.tsv`](specs/08-comandos-autocad.tsv)).
- A CSP do Tauri será ligada (spec 09).
- O DXF gravado passará de `AC1021` para `AC1032` (MT-QP-06).

## Em andamento

- Nada em andamento; o bloco B de KV segue pelo MT-KV-10, depois do ADR 0009.

## Próximo passo sugerido

1. Mantenedor: aceitar ou ajustar o ADR 0009; então MT-KV-16 (aplicar no build).
2. MT-KV-10: desenhista WebGL2 com reserva em canvas 2D (não aumenta o `.wasm`).
3. Em paralelo, quando couber: MT-UI-01 (CSP), que não depende de KV.

## Impedimentos abertos

- O GLM-5.3 no gateway esgota 16–32 mil tokens raciocinando mesmo em lotes de 4 testes:
  `{"thinking":{"type":"disabled"}}` é ignorado e `chat_template_kwargs.enable_thinking=false`
  só move o raciocínio para a resposta. Para escrever testes, usar `qwen3-coder:30b`
  (~1 mil tokens por lote, formato respeitado); para crítica, `GLM-5.2-FP8 --no-think`.

- O gateway só responde com a rede da empresa ou a VPN; fora dela, a delegação cai para
  subagente (como na parte Rust do MT-K3-12).
- **Teto do `.wasm` quase esgotado:** 463 233 de 465 000 bytes. O MT-KV-10 não mexe no
  `.wasm`; o MT-KV-11 (texto) precisa do ADR 0009 aceito.
- MT-HG-07 depende de o mantenedor instalar o `gitleaks` na máquina.

> Painel de tickets (o que existe e o que falta): [`docs/TICKETS.md`](TICKETS.md).
