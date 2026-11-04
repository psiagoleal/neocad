<!-- Caminho relativo: docs/spikes/kv-camada-gpu.md -->

# Spike MT-KV-09 — camada da GPU: WebGL2 em TypeScript ou `wgpu` em Rust

- **Data:** 2026-10-31 · **Ticket:**
  [MT-KV-09](../tickets/kv-kernel-e-o-que-se-ve.md#mt-kv-09-escolher-a-camada-da-gpu--webgl2-em-typescript-ou-wgpu-em-rust)
- **Protótipo:** ramo local `spike/kv-camada-gpu` (descartável, fora de `master`): crate
  `kernel/spike-wgpu` e bancada em `spike/kv09/`.
- **Pergunta:** sobre os blocos que o kernel prepara (MT-KV-08), qual camada desenha na
  GPU: WebGL2 chamado do TypeScript, ou `wgpu` dentro do mesmo `.wasm` do kernel?
- **Escolha:** **WebGL2 em TypeScript.** As duas camadas cumprem N1–N3 com a mesma folga;
  `wgpu` custa 1,6 MB a mais de `.wasm` sem ganho medido.

## Como foi medido

Desenho de `scripts/gerar-dxf-sintetico.mjs` com 100 mil entidades (semente 1, 20
camadas; 90 mil no espaço-modelo, 240 mil segmentos depois da divisão). Chrome 156 do
sistema, **com janela**, controlado pelo Playwright, numa RTX 3070 (driver 595.91) no
Ubuntu 26.04; o WebGL relatou
`ANGLE (NVIDIA Corporation, NVIDIA GeForce RTX 3070/PCIe/SSE2, OpenGL ES 3.2)`. Canvas
de 1600 × 900, erro de corda de um pixel da vista inteira. Três rodadas por camada;
abaixo, a mediana.

- **WebGL2:** um buffer por camada com `bufferData` do `Float32Array` que `screenBlock`
  devolve, `drawArrays(LINES)` e a vista num `uniform` (escala e deslocamento, com a
  origem do espaço já somada em `f64`).
- **`wgpu` 30.0.1:** só o backend GL (WebGL2), compilado no mesmo `.wasm` do kernel; os
  blocos passam do kernel à GPU sem atravessar para o JavaScript.

Abrir mede de `openDxf` ao primeiro quadro na tela; editar, de `addLine` ao quadro
seguinte, passando por `updateScreen` e o reenvio do bloco que mudou; pan e zoom, 240
quadros seguidos com a vista mudando a cada um.

## Números

| Medida                                     |       WebGL2 (TS) |       `wgpu` (Rust) |               Meta |
| ------------------------------------------ | ----------------: | ------------------: | -----------------: |
| Abrir: ler o DXF no kernel                 |            2,00 s |              1,94 s |                  — |
| Abrir: preparar os blocos                  |             78 ms |               62 ms |                  — |
| Abrir: enviar à GPU e primeiro quadro      |            125 ms |              109 ms |                  — |
| **Abrir, total (N1)**                      |        **2,20 s** |          **2,13 s** |              ≤ 5 s |
| **Uma linha nova até o quadro (N2)**       |        **7,5 ms** |          **4,0 ms** |           ≤ 100 ms |
| **Pan e zoom, mediana entre quadros (N3)** |       **16,7 ms** |         **16,7 ms** | ≤ 33,3 ms (30 qps) |
| Pan e zoom, p95 entre quadros              |           16,8 ms |             16,8 ms |                  — |
| Custo de CPU por quadro (mediana / p95)    |    < 0,1 / 0,1 ms |        0,2 / 0,4 ms |                  — |
| Instanciar o `.wasm`                       |              6 ms |               19 ms |                  — |
| `.wasm` (bruto / gzip)                     | 463 233 / 191 355 | 2 042 252 / 806 301 | teto 465 000 bruto |
| Cola JavaScript do `.wasm` (bruto / gzip)  |    46 348 / 8 788 |    108 466 / 17 049 |                  — |

O intervalo entre quadros é o do monitor (60 Hz) nas duas: nenhuma chega perto de gastar
um quadro inteiro. N1 é dominado pela leitura do DXF no kernel (~2 s), igual nas duas.

## Leitura

- **Desempenho não decide.** As diferenças de N1 e N2 estão dentro da variação entre
  rodadas (o envio à GPU variou de 24 a 230 ms na mesma camada). A cópia do bloco pela
  ponte, que era o argumento a favor de `wgpu`, não aparece na medida.
- **Tamanho decide.** `wgpu` multiplica o `.wasm` por 4,4: +1,58 MB bruto, +615 KB
  comprimido, que todo usuário baixa. O desenhista WebGL2 do protótipo tem ~40 linhas.
- **O texto custa o mesmo nas duas.** Pelo ADR 0008 a posição e a forma dos glifos saem
  do kernel; a camada da GPU só ganha um segundo tipo de primitiva (triângulos), poucas
  dezenas de linhas em qualquer linguagem.
- **A reserva em canvas 2D (N4) é TypeScript de todo jeito.** Com `wgpu`, haveria dois
  desenhistas em duas linguagens; com WebGL2, os dois ficam lado a lado em
  `src/lib/render/` e leem os mesmos blocos.
- **WebGPU continua aberto.** Quando entrar (ADR 0008, item 1), é outro desenhista no
  mesmo lugar, consumindo os mesmos blocos.

## Ressalvas

- Uma máquina, uma GPU, um navegador; AMD e Intel ficam presumidas (spec 00, N5). Três
  rodadas por camada.
- O custo por quadro é medido na CPU, com `gl.finish()`; o Chrome não garante que isso
  espere a GPU inteira. O que sustenta N3 é o intervalo entre quadros, e esse ficou no
  limite do monitor.
- A edição foi feita pela sessão do kernel (`addLine`), não pela interface.
- O WebView2 (Windows) e o WebKitGTK (Tauri no Linux) não foram medidos; entram no
  MT-KV-15.

## Consequência

O MT-KV-10 implementa o desenhista em WebGL2 no TypeScript, com a reserva em canvas 2D,
sobre `CadDocument.getScreenBlock`. O teto do `.wasm` (spec 06, W5) ainda precisa subir
para caber o texto do kernel (MT-KV-11), mas na casa das dezenas de KB, não de 1,6 MB:
ver o ADR que revisa o teto.
