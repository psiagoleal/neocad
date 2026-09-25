<!-- Caminho relativo: docs/TICKETS.md -->

# Tickets

> Visão geral rápida do que foi feito e do que falta. **Uma linha por ticket**: identificador
> com link para o detalhe, e uma frase curta. O detalhe mora no destino do link, não aqui.
> Atualizado **sempre** que um ticket é criado, concluído ou abandonado — skill
> `micro-ticket-planner`.

As fases seguem o [ADR 0003](adr/0003-kernel-cad-proprio.md). K1, K2 e KL estão fechadas e
são listadas por fase, não ticket a ticket; K3 está em curso e vai detalhada.

## Em aberto

- [ ] [MT-K3-10](tickets/k3-geometria-2d-edicao.md#mt-k3-10-aparar-e-estender-trimextend) —
      aparar contra arestas de corte e estender até uma fronteira.
- [ ] [MT-K3-11](tickets/k3-geometria-2d-edicao.md#mt-k3-11-concordância-e-chanfro-filletchamfer) —
      concordância e chanfro entre duas curvas.
- [ ] [MT-K3-12](tickets/k3-geometria-2d-edicao.md#mt-k3-12-expor-consulta-e-edição-na-fachada-webassembly) —
      expor consulta e edição na fachada WebAssembly; fecha a fase K3.

## Concluídos

- [x] [K1 — modelo, documento e transações](tickets/k1-modelo-documento-transacoes.md) —
      17 tickets: modelo de documento próprio, com toda mutação por transação reversível.
- [x] [K2 — DXF nativo](tickets/k2-dxf-nativo.md) — leitura e gravação determinística.
- [x] [KL — layouts de espaço-papel](tickets/kl-layouts-espaco-papel.md) — ver
      [ADR 0005](adr/0005-layouts-de-espaco-papel.md).
- [x] [MT-K3-01](tickets/k3-geometria-2d-edicao.md#mt-k3-01-mover-as-primitivas-2d-para-neocad-geometry) —
      primitivas movidas para `neocad-geometry`.
- [x] [MT-K3-02](tickets/k3-geometria-2d-edicao.md#mt-k3-02-introduzir-o-tolerance-e-converter-os-epsilons-existentes) —
      `Tolerance` e predicado de orientação exato ([ADR 0006](adr/0006-tolerancia-e-robustez-da-geometria-2d.md)).
- [x] [MT-K3-03](tickets/k3-geometria-2d-edicao.md#mt-k3-03-abaulamento-bulge-na-polilinha) —
      abaulamento na polilinha.
- [x] [MT-K3-04](tickets/k3-geometria-2d-edicao.md#mt-k3-04-avaliação-uniforme-de-curva) —
      traço `Curve`: ponto, tangente, comprimento e ponto mais próximo.
- [x] [MT-K3-05](tickets/k3-geometria-2d-edicao.md#mt-k3-05-interseção-entre-primitivas) —
      interseção entre as quatro primitivas.
- [x] [MT-K3-06](tickets/k3-geometria-2d-edicao.md#mt-k3-06-índice-espacial-e-consulta-por-região) —
      índice espacial e consulta por região.
- [x] [MT-K3-07](tickets/k3-geometria-2d-edicao.md#mt-k3-07-teste-de-acerto-e-prendimento-osnap) —
      acerto e prendimento do cursor; fecha o bloco B.
- [x] [MT-K3-08](tickets/k3-geometria-2d-edicao.md#mt-k3-08-transformações-de-entidade) —
      mover, copiar, girar, espelhar e escalar; abre o bloco C.
- [x] [MT-K3-09](tickets/k3-geometria-2d-edicao.md#mt-k3-09-paralela-offset) —
      paralela, com aparo do que fica perto demais do original.

## Abandonados

<!-- - [~] [MT-0](plano.md#mt-0) — <frase curta> · motivo: <uma frase> -->
