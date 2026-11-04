<!-- Caminho relativo: docs/specs/kp5-cotas.md -->

# KP5 — Cotas

- **Estado:** Esqueleto — critérios EARS a fechar quando a fase abrir
- **Fase:** KP · **ADRs:** 0007 · **Transversais e relacionadas:** 00, 02, 04

## Problema e resultado esperado

Sem cota não há desenho de engenharia. O modelo não representa cota, e cotas lidas de arquivo viram perda ao salvar.

## Escopo

Cotas linear, alinhada, angular, de raio, de diâmetro, de comprimento de arco, por ordenada, em série e de linha base; marca de centro; estilos de cota com as variáveis principais; sobreposição de estilo por cota; associatividade à geometria; tolerância geométrica.

## Fora de escopo

Restrições paramétricas por cota (`DIMCONSTRAINT`), que pertencem a K4.

## Comandos envolvidos

Grafia oficial de cada idioma, de [`08-comandos-autocad.tsv`](08-comandos-autocad.tsv).

| comando EN        | comando PT-BR     | apelido      |
| ----------------- | ----------------- | ------------ |
| `DIMSTYLE`        | `ESTILOCOTA`      | `D`          |
| `DIMANGULAR`      | `COTAANG`         | `DAN`        |
| `DIMARC`          | `COTAARCO`        | `DAR`        |
| `DIMBASELINE`     | `COTALINBASE`     | `DBA`        |
| `DIMCONTINUE`     | `COTASERIE`       | `DCO`        |
| `DIMDIAMETER`     | `COTADIAMETRO`    | `DDI`        |
| `DIMRADIUS`       | `COTARAIOS`       | `DRA`        |
| `DIMORDINATE`     | `COTAORDEN`       | `DOR`        |
| `DIMCENTER`       | `COTACENTRO`      | `DCE`        |
| `DIMEDIT`         | `EDITARCOTA`      | `DED`        |
| `DIMJOGGED`       | `COTACOMDESVIO`   | `DJO`, `JOG` |
| `DIMJOGLINE`      | `COTALINDESVIO`   | `DJL`        |
| `DIMOVERRIDE`     | `SOBRECOTA`       | `DOV`        |
| `DIMREASSOCIATE`  | `REASSOCIARCOTA`  | `DRE`        |
| `DIMDISASSOCIATE` | `COTADESASSOCIAR` | `DDA`        |
| `TOLERANCE`       | `TOLERANCIA`      | `TOL`        |

## Restrições já tomadas

- Toda entidade nova é lida e gravada em DXF no mesmo ticket (00, T5).
- Todo comando aceita os dois idiomas e o apelido (spec 08).

## Questões em aberto

- `DIMLINEAR` e `DIMALIGNED` não aparecem na página de atalhos; buscar os nomes nos dois idiomas na documentação oficial da Autodesk antes de fechar a spec.
- O bloco é grande; avaliar quebra em duas specs (geometria da cota × estilos e associatividade).
