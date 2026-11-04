<!-- Caminho relativo: docs/spikes/kv-espelhamento.md -->

# Spike MT-KV-03 — espelhar o kernel no modelo do upstream

- **Data:** 2026-10-30 · **Ticket:**
  [MT-KV-03](../tickets/kv-kernel-e-o-que-se-ve.md#mt-kv-03-spike--espelhar-o-kernel-no-modelo-do-upstream)
- **Protótipo:** ramo local `spike/kv-espelhamento` (descartável, fora de `master`).
- **Pergunta:** se cada mudança do kernel for aplicada também ao `data-model` do upstream,
  que continua desenhando, as metas da [spec 07](../specs/07-kernel-e-o-que-se-ve.md)
  fecham?

## Como foi medido

Desenho gerado por `scripts/gerar-dxf-sintetico.mjs` (semente 1, 20 camadas), aberto pelo
aplicativo de produção (`pnpm build && pnpm preview`) num Chrome 156 controlado pelo
Playwright, numa máquina com Ryzen 5 3500X (6 núcleos). Cada edição foi feita no kernel
pelo `cad-document` e repetida no upstream com `appendEntity`, mudança de pontos seguida
de `triggerModifiedEvent` e `erase`. O tempo de cada edição vai do início da chamada até
o segundo quadro de animação seguinte, que só chega depois de o upstream processar o
evento. É o mesmo critério do [MT-KV-04](kv-render-kernel.md).

O gerador foi corrigido depois da medida (o espaço-papel recebia só polilinha e texto).
O modelo medido tinha 20 mil linhas, círculos e arcos e 15 mil polilinhas e textos, em vez
de 18 mil de cada. A diferença não muda a conclusão.

## Números

| Medida                                          | 10 mil entidades |      100 mil entidades | Meta (spec 07) |
| ----------------------------------------------- | ---------------: | ---------------------: | -------------: |
| Abrir: kernel lê o arquivo                      |           0,15 s |                  2,1 s |              — |
| Abrir: upstream termina                         |            2,0 s |                  8,9 s |            5 s |
| Criar uma linha (kernel + upstream + quadro)    |          ~220 ms |                 ~2,2 s |         100 ms |
| Mover uma linha                                 |          ~280 ms |                 ~2,2 s |         100 ms |
| Apagar uma linha                                |          ~220 ms |                 ~2,3 s |         100 ms |
| Só o kernel, por edição                         |           < 1 ms |                 < 1 ms |              — |
| `listEntities()` do kernel                      |            42 ms |                 355 ms |              — |
| Converter tudo do kernel em objetos do upstream |            42 ms |                 496 ms |              — |
| Esvaziar o espaço-modelo do upstream            |            1,2 s | não terminou em 13 min |              — |

O custo de cada edição está no upstream: a edição no kernel leva menos de 1 ms, e o
quadro seguinte a um evento do upstream leva cerca de 2,2 s com 100 mil entidades (e
cerca de 31 ms sem evento). O tempo cresce com o tamanho do desenho, não com o tamanho
da edição.

## Fidelidade

O `data-model` do upstream tem classe para todo tipo que o kernel modela hoje: reta,
círculo, arco, polilinha com bojo, texto e janela de layout. Converter as 90 mil entidades
do espaço-modelo não recusou nenhuma. Não verificado no spike: janelas de layout e
propriedades de camada espelhadas. O handoff de K2 já registrou que o leitor do upstream
não separa as entidades de espaço-papel ao abrir.

## O que não fecha

- **Edição:** 2,2 s contra 100 ms, 22 vezes acima, e o custo é interno ao upstream.
- **Abertura:** 8,9 s contra 5 s, mesmo usando a leitura do próprio upstream.
- **Ressincronizar:** esvaziar e repovoar o modelo do upstream é inviável (o esvaziamento
  não terminou). Qualquer divergência entre os dois modelos só se corrige reabrindo.
- **Os dois modelos continuam vivos**, que é o problema que a spec 07 quer eliminar. A
  barra de comandos do upstream continuaria editando o modelo dele.
- **Lacuna comum aos dois mecanismos:** a fachada não tem consulta de uma entidade pelo
  id; para saber o que mudou, hoje só existe `listEntities()` (355 ms com 100 mil).

## Como reproduzir

No ramo `spike/kv-espelhamento`:
`NEOCAD_SINTETICO_N=100000 pnpm exec playwright test e2e/zz-spike-espelho.e2e.ts --reporter=line`.
