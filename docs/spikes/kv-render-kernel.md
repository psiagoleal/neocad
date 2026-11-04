<!-- Caminho relativo: docs/spikes/kv-render-kernel.md -->

# Spike MT-KV-04 — desenhar a partir do kernel

- **Data:** 2026-10-30 · **Ticket:**
  [MT-KV-04](../tickets/kv-kernel-e-o-que-se-ve.md#mt-kv-04-spike--desenhar-a-partir-do-kernel)
- **Protótipo:** ramo local `spike/kv-render-kernel` (descartável, fora de `master`).
- **Pergunta:** se a tela for desenhada direto do que o kernel contém, as metas da
  [spec 07](../specs/07-kernel-e-o-que-se-ve.md) fecham, e quanto custa chegar à paridade
  visual com o upstream?

## Como foi medido

Mesmo desenho, máquina, navegador e critério de tempo do
[MT-KV-03](kv-espelhamento.md). O protótipo é um canvas 2D sobreposto ao viewer, com
cerca de 70 linhas:

- Constrói um `Path2D` em coordenadas de mundo por camada, a partir de `listEntities()`.
- Faz pan e zoom só mudando a transformação.
- Desenha os textos com `fillText`, pulando os menores que 2 px.
- Numa edição, refaz o caminho da camada afetada e redesenha tudo.

Polilinha com bojo foi desenhada reta, e não há cor por camada, tipo de linha nem
espessura.

## Números

| Medida                                   | 10 mil entidades | 100 mil entidades | Meta (spec 07) |
| ---------------------------------------- | ---------------: | ----------------: | -------------: |
| Abrir: kernel lê o arquivo               |           0,15 s |             2,1 s |              — |
| `listEntities()` + caminhos + 1º quadro  |            73 ms |            0,64 s |              — |
| Abrir, total até a tela                  |          ~0,25 s |            ~2,8 s |            5 s |
| Criar uma linha (kernel + tela + quadro) |            33 ms |            115 ms |         100 ms |
| Mover uma linha                          |            33 ms |            122 ms |         100 ms |
| Apagar uma linha                         |            33 ms |            118 ms |         100 ms |
| Pan, por quadro, com texto               |            33 ms |            196 ms |              — |
| Pan, por quadro, sem texto               |            33 ms |            153 ms |              — |

33 ms é o piso do critério (dois quadros a 60 Hz). Com 100 mil, a edição gasta cerca de
80 ms acima do piso. A causa é refazer o caminho de uma camada inteira (4,5 mil entidades)
e redesenhar as 90 mil.

## O que não fecha, e o que fecharia

- **Edição:** 115–122 ms contra 100 ms. Perto, e o protótipo não faz nada para
  economizar. Desenhar a camada alterada numa camada própria, ou só a região afetada
  (o kernel já tem `entitiesInBox` sobre índice espacial), deve trazer abaixo da meta;
  falta medir.
- **Pan:** 5 a 6 quadros por segundo com 100 mil no canvas 2D. Fluidez exige recorte pela
  vista (desenhar só o que `entitiesInBox` devolve), cache em bitmap durante o arraste, ou
  WebGL. O three.js já é dependência do upstream.
- **Mesma lacuna do MT-KV-03:** falta consultar uma entidade pelo id. No protótipo, a
  entidade editada foi montada do lado JavaScript, sem ida ao kernel.

## Custo estimado da paridade visual nas entidades modeladas

O que o upstream faz e o protótipo não faz:

- cor por camada e por entidade;
- tipo de linha e espessura;
- bojo de polilinha;
- texto com fonte e alinhamento do DXF (o upstream rasteriza SHX e TTF em worker);
- janelas de layout recortando o modelo;
- fundo claro e escuro;
- ajuste à tela;
- realce de seleção (K5).

Estimativa grosseira: de duas a quatro semanas de um desenvolvedor, para chegar ao que o
upstream mostra hoje dessas entidades. O item mais caro é texto fiel, seguido de WebGL ou
cache para o pan. Entidades que o kernel ainda não modela (cota, hachura, bloco com
atributo) continuariam pelo upstream até a fase KP. Isso exige convivência dos dois
desenhos na mesma tela, o que a spec 07 já prevê.

## Como reproduzir

No ramo `spike/kv-render-kernel`:
`NEOCAD_SINTETICO_N=100000 pnpm exec playwright test e2e/zz-spike-render.e2e.ts --reporter=line`.
