<!-- Caminho relativo: docs/TICKETS.md -->

# Tickets

> Visão geral rápida do que foi feito e do que falta. **Uma linha por ticket**: identificador
> com link para o detalhe, e uma frase curta. O detalhe mora no destino do link, não aqui.
> Atualizado **sempre** que um ticket é criado, concluído ou abandonado — skill
> `micro-ticket-planner`.

As fases seguem os ADRs [0003](adr/0003-kernel-cad-proprio.md) e
[0007](adr/0007-fase-de-prancha-de-engenharia.md); o aceite de cada ticket é a
[spec](specs/README.md) correspondente. K1, K2 e KL estão fechadas e são listadas por fase;
as demais vão ticket a ticket. Ordem: HG e QP em paralelo ao fechamento de K3; depois KV;
UI começa pelo que não depende do ADR 0008.

## Em aberto

- [ ] [MT-HG-07](tickets/hg-higiene.md#mt-hg-07-varredura-de-segredos-local-antes-do-commit) —
      hook `gitleaks` — bloqueado por `gitleaks` não instalado na máquina.
- [ ] [MT-QP-01](tickets/qp-provas-das-specs.md#mt-qp-01-ida-e-volta-dxf-por-variante-de-geometria-t5) —
      variante nova sem leitura e gravação não compila.
- [ ] [MT-QP-02](tickets/qp-provas-das-specs.md#mt-qp-02-guarda-de-tolerância-e-de-licenças-do-kernel-t6-t7) —
      script em `make kernel-check` e na CI.
- [ ] [MT-QP-03](tickets/qp-provas-das-specs.md#mt-qp-03-origem-das-fixtures-t10) —
      `ORIGEM.md` e teste que exige a linha.
- [ ] [MT-QP-04](tickets/qp-provas-das-specs.md#mt-qp-04-e2e-do-aviso-de-perda-ao-salvar-t4) —
      diálogo com tipos e quantidades; cancelar não grava.
- [ ] [MT-QP-05](tickets/qp-provas-das-specs.md#mt-qp-05-teste-direto-da-ordem-das-abas-03-l1) —
      `in_tab_order` com empate desfeito pelo nome.
- [ ] [MT-QP-06](tickets/qp-provas-das-specs.md#mt-qp-06-gravar-dxf-ac1032-02-x7) —
      versão gravada passa a 2018.
- [ ] [MT-QP-07](tickets/qp-provas-das-specs.md#mt-qp-07-corpus-dxf-de-origem-pública-por-versão-02-x8-review-g4) —
      R12 a AC1032 lidos em teste.
- [ ] [MT-QP-08](tickets/qp-provas-das-specs.md#mt-qp-08-limite-de-tamanho-do-wasm-06-w5) —
      build falha acima de 465 000 bytes.
- [ ] [MT-QP-09](tickets/qp-provas-das-specs.md#mt-qp-09-testes-de-propriedade-da-geometria-review-b4) —
      interseção, paralela e transformação.
- [ ] [MT-KV-02](tickets/kv-kernel-e-o-que-se-ve.md#mt-kv-02-gerador-de-desenho-sintético-grande) —
      100 mil entidades sem arquivo de terceiros.
- [ ] [MT-KV-03](tickets/kv-kernel-e-o-que-se-ve.md#mt-kv-03-spike--espelhar-o-kernel-no-modelo-do-upstream) —
      mede a opção de espelhamento.
- [ ] [MT-KV-04](tickets/kv-kernel-e-o-que-se-ve.md#mt-kv-04-spike--desenhar-a-partir-do-kernel) —
      mede a opção de desenho próprio.
- [ ] [MT-KV-05](tickets/kv-kernel-e-o-que-se-ve.md#mt-kv-05-adr-0008--mecanismo-de-kv) —
      escolhe com base nos dois relatórios.
- [ ] [MT-UI-01](tickets/ui-interface.md#mt-ui-01-ligar-a-csp-e-servir-a-mesma-política-no-vite) —
      spec 09, S1, S2, S5.
- [ ] [MT-UI-02](tickets/ui-interface.md#mt-ui-02-e2e-que-falha-em-violação-de-csp) —
      spec 09, S3, S3b, S6.
- [ ] [MT-UI-03](tickets/ui-interface.md#mt-ui-03-conferir-a-csp-no-app-empacotado) —
      spec 09, S4, em Linux e Windows.
- [ ] [MT-UI-04](tickets/ui-interface.md#mt-ui-04-resolvedor-de-comandos-bilíngue) —
      nomes EN/PT, apelidos e prefixos (spec 08).
- [ ] [MT-UI-05](tickets/ui-interface.md#mt-ui-05-catálogo-de-comandos-bilíngue) —
      `Ajuda > Comandos CAD` nos dois idiomas (C11).

## Concluídos

- [x] [MT-KV-01](tickets/kv-kernel-e-o-que-se-ve.md#mt-kv-01-e2e-que-prova-a-perda-da-linha-desenhada-pelo-upstream) —
      `e2e/kernel-view.e2e.ts` prova a perda (V4), marcado `test.fail()` até KV.
- [x] [MT-K3-12](tickets/k3-geometria-2d-edicao.md#mt-k3-12-expor-consulta-e-edição-na-fachada-webassembly) —
      consulta e edição na fachada WebAssembly e no serviço `cad-document`; fecha a fase K3 (spec 06).
- [x] [MT-K3-11](tickets/k3-geometria-2d-edicao.md#mt-k3-11-concordância-e-chanfro-filletchamfer) —
      concordância (reta e arco) e chanfro (retas), pelos pontos indicados (spec 05, E10–E13).
- [x] [MT-K3-10](tickets/k3-geometria-2d-edicao.md#mt-k3-10-aparar-e-estender-trimextend) —
      aparar e estender reta, arco, circunferência e polilinha (spec 05, E7–E9).
- [x] [MT-K3-13](tickets/k3-geometria-2d-edicao.md#mt-k3-13-recusar-fator-de-escala-zero-negativo-ou-não-finito) —
      fator ≤ 0 recusado, como no AutoCAD (spec 05, E1c).
- [x] [MT-HG-03](tickets/hg-higiene.md#mt-hg-03-atualizar-o-readme-e-consolidar-o-roteiro) —
      README de hoje; roteiro pelos ADRs 0003 e 0007.
- [x] [MT-HG-06](tickets/hg-higiene.md#mt-hg-06-pôr-o-cabeçalho-de-caminho-relativo-nos-arquivos-que-faltam) —
      15 arquivos sem o cabeçalho do §4.
- [x] [MT-HG-05](tickets/hg-higiene.md#mt-hg-05-decidir-o-codex-e-a-regra-do-changelog) —
      ignorar `.codex/`; uma linha por mudança.
- [x] [MT-HG-04](tickets/hg-higiene.md#mt-hg-04-corrigir-o-mapa-de-arquitetura) —
      arestas reais entre crates e onde a GPL está de fato.
- [x] [MT-HG-02](tickets/hg-higiene.md#mt-hg-02-enxugar-o-handoff) —
      estado corrente curto; histórico em `handoff-arquivo.md`.
- [x] [MT-HG-01](tickets/hg-higiene.md#mt-hg-01-corrigir-as-ilhas-do-agentsmd) —
      stack e comandos reais; fragmento colado e numeração.
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
