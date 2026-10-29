<!-- Caminho relativo: docs/specs/kp2-geometria-complementar.md -->

# KP2 — Geometria complementar

- **Estado:** Esqueleto — critérios EARS a fechar quando a fase abrir
- **Fase:** KP · **ADRs:** 0007 · **Transversais e relacionadas:** 00, 02, 04, 05

## Problema e resultado esperado

Elipse, spline e ponto são comuns em desenho de engenharia e hoje viram perda ao salvar. A escala não uniforme também espera a elipse.

## Escopo

Elipse e arco elíptico, spline, ponto (com estilo de exibição), linha infinita e semirreta; avaliação, interseção e prendimento para cada uma (spec 04); edição de polilinha; divisão e medição ao longo de curva.

## Fora de escopo

NURBS 3D (K7).

## Comandos envolvidos

Grafia oficial de cada idioma, de [`08-comandos-autocad.tsv`](08-comandos-autocad.tsv).

| comando EN  | comando PT-BR  | apelido |
| ----------- | -------------- | ------- |
| `ELLIPSE`   | `ELIPSE`       | `EL`    |
| `SPLINE`    | `SPLINE`       | `SPL`   |
| `SPLINEDIT` | `EDITARSPLINE` | `SPE`   |
| `POINT`     | `PONTO`        | `PO`    |
| `XLINE`     | `LINHAINF`     | `XL`    |
| `DONUT`     | `ANEL`         | `DO`    |
| `DIVIDE`    | `DIVIDIR`      | `DIV`   |
| `MEASURE`   | `MEDIDA`       | `ME`    |
| `PEDIT`     | `EDITARP`      | `PE`    |

## Restrições já tomadas

- Toda entidade nova é lida e gravada em DXF no mesmo ticket (00, T5).
- Todo comando aceita os dois idiomas e o apelido (spec 08).

## Questões em aberto

- Precisão da interseção com spline: iterativa, com que tolerância (ADR 0006).
