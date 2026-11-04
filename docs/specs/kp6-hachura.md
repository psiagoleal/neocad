<!-- Caminho relativo: docs/specs/kp6-hachura.md -->

# KP6 — Hachura

- **Estado:** Esqueleto — critérios EARS a fechar quando a fase abrir
- **Fase:** KP · **ADRs:** 0007 · **Transversais e relacionadas:** 00, 02, 04

## Problema e resultado esperado

Cortes, materiais e áreas usam hachura. O modelo não a representa, e K3 adiou as booleanas 2D justamente por falta deste consumidor.

## Escopo

Hachura por padrão, sólida e em gradiente; contorno por seleção e por ponto interno; ilhas; associatividade ao contorno; edição; contorno e região a partir de área fechada; booleanas 2D que isso exige.

## Fora de escopo

Hachura com padrão definido pelo usuário em arquivo `.pat` próprio (avaliar depois).

## Comandos envolvidos

Grafia oficial de cada idioma, de [`08-comandos-autocad.tsv`](08-comandos-autocad.tsv).

| comando EN  | comando PT-BR | apelido   |
| ----------- | ------------- | --------- |
| `HATCH`     | `HACHURA`     | `H`, `BH` |
| `HATCHEDIT` | `EDITARHACH`  | `HE`      |
| `GRADIENT`  | `GRADIENTE`   | `GD`      |
| `BOUNDARY`  | `LIMITE`      | `BO`      |
| `REGION`    | `REGIAO`      | `REG`     |

## Restrições já tomadas

- Toda entidade nova é lida e gravada em DXF no mesmo ticket (00, T5).
- Todo comando aceita os dois idiomas e o apelido (spec 08).

## Questões em aberto

- Padrões de hachura a distribuir e sua licença (T10).
