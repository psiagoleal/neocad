<!-- Caminho relativo: docs/specs/README.md -->

# Specs do NeoCAD

A spec é o **contrato** do resultado de uma feature: o que conta como pronto. O código é
derivado dela e o teste é a spec em forma executável. Conforme a skill
`spec-como-contrato`:

- **Spec não é plano.** A sequência de trabalho mora em `docs/tickets/`; a spec não muda
  quando o plano muda.
- **Spec não é ADR.** Ela cita a decisão de arquitetura, não a rediscute.
- **Todo critério de aceite é escrito em EARS** (`DEVE`, nunca "deveria") e tem um
  **comando de prova**. Critério sem prova não vira ticket.
- **Mudou o comportamento, a spec muda no mesmo commit.** Spec desatualizada é pior que
  spec inexistente.

O NeoCAD é um CAD open-source para **desenho de projeto de engenharia**. Toda spec é
lida contra esse objetivo: o entregável do usuário é a prancha, e quem vem do AutoCAD
deve encontrar os mesmos comandos, nos dois idiomas
([ADR 0007](../adr/0007-fase-de-prancha-de-engenharia.md)).

## Estados

| Estado        | Significado                                                                           |
| ------------- | ------------------------------------------------------------------------------------- |
| **Vigente**   | Descreve comportamento existente; as provas passam hoje.                              |
| **Aprovada**  | Comportamento a construir; critérios e provas fechados, pronta para virar ticket.     |
| **Esqueleto** | Problema, escopo e fronteiras fixados; critérios EARS ficam para quando a fase abrir. |

## Índice

| Spec                                                            | Fase  | Estado                     |
| --------------------------------------------------------------- | ----- | -------------------------- |
| [00 — Requisitos transversais](00-transversal.md)               | todas | Vigente                    |
| [01 — Documento e transações](01-documento-e-transacoes.md)     | K1    | Vigente                    |
| [02 — DXF: leitura e gravação](02-dxf.md)                       | K2    | Vigente, com item aprovado |
| [03 — Layouts de espaço-papel](03-layouts.md)                   | KL    | Vigente                    |
| [04 — Consulta geométrica](04-consulta-geometrica.md)           | K3    | Vigente                    |
| [05 — Edição 2D](05-edicao-2d.md)                               | K3    | Vigente + aprovada         |
| [06 — Fachada WebAssembly](06-fachada-wasm.md)                  | K3    | Aprovada                   |
| [07 — O kernel é o que se vê](07-kernel-e-o-que-se-ve.md)       | KV    | Aprovada                   |
| [08 — Linha de comando](08-linha-de-comando.md)                 | UI/UX | Aprovada                   |
| [09 — Segurança do webview](09-seguranca-do-webview.md)         | UI/UX | Aprovada                   |
| [10 — Comandos de desenho básicos](10-desenho-basico.md)        | UI/UX | Esqueleto                  |
| [KP1 — Propriedades de desenho](kp1-propriedades-de-desenho.md) | KP    | Esqueleto                  |
| [KP2 — Geometria complementar](kp2-geometria-complementar.md)   | KP    | Esqueleto                  |
| [KP3 — Texto](kp3-texto.md)                                     | KP    | Esqueleto                  |
| [KP4 — Blocos e atributos](kp4-blocos.md)                       | KP    | Esqueleto                  |
| [KP5 — Cotas](kp5-cotas.md)                                     | KP    | Esqueleto                  |
| [KP6 — Hachura](kp6-hachura.md)                                 | KP    | Esqueleto                  |
| [KP7 — Plotagem e PDF](kp7-plotagem.md)                         | KP    | Esqueleto                  |

Dados versionados junto: [`08-comandos-autocad.tsv`](08-comandos-autocad.tsv) — 225
apelidos com o nome oficial do comando em inglês e em português.

## Formato

```markdown
<!-- Caminho relativo: docs/specs/NN-nome.md -->

# NN — Título

- **Estado:** Vigente | Aprovada | Esqueleto
- **Fase:** … · **ADRs:** … · **Transversais:** 00

## Problema e resultado esperado

## Escopo

## Fora de escopo

## Restrições e decisões já tomadas

## Critérios de aceite (EARS)

| id | critério | prova |

## Casos de erro

## Requisitos não-funcionais

## Questões em aberto
```
