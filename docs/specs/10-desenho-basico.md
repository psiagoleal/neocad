<!-- Caminho relativo: docs/specs/10-desenho-basico.md -->

# 10 — Comandos de desenho básicos

- **Estado:** Esqueleto — critérios EARS a fechar quando a fase abrir
- **Fase:** UI/UX · **ADRs:** 0007 · **Transversais e relacionadas:** 05, 07, 08

## Problema e resultado esperado

O usuário precisa criar a geometria que o kernel já modela — reta, circunferência, arco, polilinha — pela linha de comando, com os pontos pedidos no prompt, prendimento e pré-visualização, como no AutoCAD.

## Escopo

Criação interativa de reta, circunferência (centro e raio, diâmetro, 2 e 3 pontos), arco (3 pontos e variantes usuais), polilinha com segmentos de arco, retângulo e polígono; entrada de coordenada absoluta, relativa (`@x,y`) e polar (`@d<a`); apagar.

## Fora de escopo

Elipse, spline, ponto, linha infinita (KP2). Rastreamento polar, ortogonal e entrada dinâmica (spec própria de UI/UX).

## Comandos envolvidos

Grafia oficial de cada idioma, de [`08-comandos-autocad.tsv`](08-comandos-autocad.tsv).

| comando EN | comando PT-BR | apelido |
| ---------- | ------------- | ------- |
| `LINE`     | `LINHA`       | `L`     |
| `CIRCLE`   | `CIRCULO`     | `C`     |
| `ARC`      | `ARC`         | `A`     |
| `PLINE`    | `PLINHA`      | `PL`    |
| `RECTANG`  | `RETANGULO`   | `REC`   |
| `POLYGON`  | `POLIGONO`    | `POL`   |
| `ERASE`    | `APAGAR`      | `E`     |

## Restrições já tomadas

- Toda entidade nova é lida e gravada em DXF no mesmo ticket (00, T5).
- Todo comando aceita os dois idiomas e o apelido (spec 08).

## Questões em aberto

- Variantes de `ARC` e `CIRCLE` que entram primeiro.
- Formato de coordenada com vírgula decimal (`1,5`) em instalação pt-BR: o AutoCAD usa ponto como decimal e vírgula como separador de coordenadas.
