<!-- Caminho relativo: docs/specs/04-consulta-geometrica.md -->

# 04 — Consulta geométrica

- **Estado:** Vigente
- **Fase:** K3, blocos A e B (concluídos) · **ADRs:** 0006 · **Transversais:** 00 (T6)

## Problema e resultado esperado

Um cursor de CAD prende em ponto notável e uma seleção sabe o que está sob o ponto. Sem
isso não há precisão, e sem precisão não há desenho de engenharia. O kernel responde,
sobre as quatro primitivas (reta, circunferência, arco, polilinha com abaulamento): onde
as curvas se cruzam, o que está numa região, o que está sob o cursor e onde o cursor deve
prender.

## Escopo

Avaliação de curva (ponto, tangente, comprimento, ponto mais próximo); interseção entre as
primitivas; índice espacial e consulta por região; teste de acerto; prendimento nos modos
extremidade, meio, centro, quadrante, interseção, perpendicular, tangente e mais próximo.

## Fora de escopo

Desenho na tela, marcadores de prendimento, rastreamento polar e ortogonal (UI/UX, sobre
a spec 07). Elipse e spline (KP2).

## Restrições e decisões já tomadas

- ADR 0006: tolerância única e predicado de orientação exato.
- Prioridade do prendimento: **modo primeiro**, distância depois, então índice da curva e
  coordenadas — o modo "mais próximo" nunca esconde os demais.

## Critérios de aceite (EARS)

| id  | critério                                                                                                                                 | prova                                                                         |
| --- | ---------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| G1  | A interseção entre quaisquer duas primitivas DEVE devolver os pontos de cruzamento e de tangência, dentro da tolerância.                 | `cargo test --manifest-path kernel/Cargo.toml -p neocad-geometry intersect::` |
| G2  | A consulta por região DEVE devolver exatamente o conjunto de candidatos de uma varredura linear, na ordem de entrada.                    | `cargo test --manifest-path kernel/Cargo.toml -p neocad-geometry spatial::`   |
| G3  | O teste de acerto DEVE medir a distância ao traço, e não à caixa envolvente.                                                             | `cargo test --manifest-path kernel/Cargo.toml -p neocad-geometry snap::`      |
| G4  | QUANDO há vários pontos notáveis no raio de captura, o prendimento DEVE escolher pela prioridade de modo antes da distância.             | `cargo test --manifest-path kernel/Cargo.toml -p neocad-geometry snap::`      |
| G5  | O prendimento perpendicular DEVE ser oferecido só quando o pé cai dentro do segmento; o tangente, só a partir de fora da circunferência. | `cargo test --manifest-path kernel/Cargo.toml -p neocad-geometry snap::`      |
| G6  | O sentido de giro de três pontos DEVE ser decidido pelo predicado exato, inclusive em pontos quase colineares.                           | `cargo test --manifest-path kernel/Cargo.toml -p neocad-geometry predicate::` |

## Requisitos não-funcionais

A consulta por região não varre o desenho inteiro; o limite de tempo será fixado em 00
depois do spike de KV.
