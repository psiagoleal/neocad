<!-- Caminho relativo: docs/specs/kp1-propriedades-de-desenho.md -->

# KP1 — Propriedades de desenho

- **Estado:** Esqueleto — critérios EARS a fechar quando a fase abrir
- **Fase:** KP · **ADRs:** 0007 · **Transversais e relacionadas:** 00, 02

## Problema e resultado esperado

Desenho de engenharia depende de unidades e precisão declaradas, tipos de linha (tracejado, traço-ponto), espessuras e escala de tipo de linha. Hoje o desenho não tem unidade, a camada guarda o tipo de linha só pelo nome e a entidade não tem tipo de linha, espessura nem escala próprios.

## Escopo

Unidades do desenho (`INSUNITS`) e precisão linear e angular; tabela de tipos de linha (`LTYPE`) com padrões; escala global e por entidade; espessura por entidade; propriedades por entidade com `BYLAYER`, `BYBLOCK` e valor explícito; cor verdadeira; consulta de propriedades e medição.

## Fora de escopo

Tipos de linha complexos com forma ou texto embutido (avaliar à parte). Estilos de plotagem (KP7).

## Comandos envolvidos

Grafia oficial de cada idioma, de [`08-comandos-autocad.tsv`](08-comandos-autocad.tsv).

| comando EN    | comando PT-BR  | apelido          |
| ------------- | -------------- | ---------------- |
| `UNITS`       | `UNIDADES`     | `UN`             |
| `LINETYPE`    | `TIPOLINHA`    | `LT`             |
| `LTSCALE`     | `ESCALATL`     | `LTS`            |
| `LWEIGHT`     | `ESPLINHA`     | `LW`             |
| `COLOR`       | `COR`          | `COL`            |
| `PROPERTIES`  | `PROPRIEDADES` | `PR`, `CH`, `MO` |
| `MATCHPROP`   | `MATCHPROP`    | `MA`             |
| `LAYER`       | `CAMADA`       | `LA`             |
| `LIST`        | `LISTA`        | `LI`             |
| `DIST`        | `DIST`         | `DI`             |
| `ID`          | `ID`           | `ID`             |
| `AREA`        | `AREA`         | `AA`             |
| `MEASUREGEOM` | `MEDIRGEOM`    | `MEA`            |

## Restrições já tomadas

- Toda entidade nova é lida e gravada em DXF no mesmo ticket (00, T5).
- Todo comando aceita os dois idiomas e o apelido (spec 08).

## Questões em aberto

- Tipos de linha padrão a oferecer num desenho novo, e de onde vêm suas definições (licença: T10).
