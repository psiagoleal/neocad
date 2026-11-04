<!-- Caminho relativo: docs/specs/kp7-plotagem.md -->

# KP7 — Plotagem e PDF

- **Estado:** Esqueleto — critérios EARS a fechar quando a fase abrir
- **Fase:** KP · **ADRs:** 0007 · **Transversais e relacionadas:** 00, 03

## Problema e resultado esperado

A prancha só vira entregável quando sai em escala, com as espessuras de pena certas, em PDF ou em papel.

## Escopo

Plotagem de layout e de janela do modelo em escala; configuração de página (já modelada em KL) aplicada; tabela de estilo de plotagem por cor (`CTB`) e nomeada (`STB`); pré-visualização; exportação para PDF vetorial; viewports e troca entre espaço-modelo e papel.

## Fora de escopo

Conjuntos de folhas (`SHEETSET`) e pacote de transmissão (`ETRANSMIT`).

## Comandos envolvidos

Grafia oficial de cada idioma, de [`08-comandos-autocad.tsv`](08-comandos-autocad.tsv).

| comando EN  | comando PT-BR | apelido |
| ----------- | ------------- | ------- |
| `PLOT`      | `PLOTAR`      | `PRINT` |
| `PREVIEW`   | `VISUALIZAR`  | `PRE`   |
| `EXPORTPDF` | `EXPORTARPDF` | `EPDF`  |
| `LAYOUT`    | `LAYOUT`      | `LO`    |
| `MVIEW`     | `CVISTA`      | `MV`    |
| `MSPACE`    | `ESPACOM`     | `MS`    |
| `PSPACE`    | `ESPACOP`     | `PS`    |

## Restrições já tomadas

- Toda entidade nova é lida e gravada em DXF no mesmo ticket (00, T5).
- Todo comando aceita os dois idiomas e o apelido (spec 08).

## Questões em aberto

- Biblioteca de geração de PDF e sua licença.
- Impressão direta pelo sistema operacional nesta fase ou só PDF.
