<!-- Caminho relativo: docs/specs/02-dxf.md -->

# 02 — DXF: leitura e gravação

- **Estado:** Vigente, com o critério X7 aprovado (a construir)
- **Fase:** K2 (concluída) · **ADRs:** 0003, 0004, 0005 · **Transversais:** 00 (T2–T5, T10)

## Problema e resultado esperado

O ciclo abrir → editar → salvar precisa fechar sem depender do parser do upstream, que só
roda em navegador. A crate `neocad-io` lê e grava DXF a partir da especificação pública,
sem dependência de terceiros.

## Escopo

Seções `HEADER`, `TABLES` (camadas, estilos de texto, registros de bloco), `BLOCKS`,
`ENTITIES` e `OBJECTS` (layouts). Entidades: `LINE`, `CIRCLE`, `ARC`, `LWPOLYLINE`,
`POLYLINE` antiga com `VERTEX`/`SEQEND`, `TEXT`, `VIEWPORT`. Relatório do que foi lido e
não representado.

## Fora de escopo

DWG (fase K6). Entidades de anotação, blocos com atributos e hachura entram por KP, cada
uma com leitura e gravação (T5).

## Restrições e decisões já tomadas

- Gravação em UTF-8, com fim de linha `CRLF`, como o AutoCAD.
- A versão gravada passa de `AC1021` (2007) para **`AC1032` (2018)** por decisão de
  2026-10-29 (X7): é o formato nativo do AutoCAD 2018 ao 2026.

## Critérios de aceite (EARS)

| id  | critério                                                                                                                             | prova                                                                                                        |
| --- | ------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------ |
| X1  | QUANDO um DXF com as entidades do escopo é lido e gravado, a releitura DEVE produzir o mesmo documento.                              | `cargo test --manifest-path kernel/Cargo.toml -p neocad-io --test round_trip`                                |
| X2  | QUANDO o arquivo contém entidade fora do escopo, a leitura DEVE concluir e o relatório DEVE trazer o tipo e a quantidade.            | `cargo test --manifest-path kernel/Cargo.toml -p neocad-io dxf::report` + `pnpm test:e2e -- kernel-document` |
| X3  | QUANDO uma polilinha antiga (`POLYLINE`/`VERTEX`) é lida, ela DEVE virar polilinha do modelo, e não contar como perda.               | `pnpm test:e2e -- dxf-constructs`                                                                            |
| X4  | QUANDO um bloco tem entidades, a leitura DEVE trazê-las para a definição do bloco, mesmo que o upstream falhe ao abrir o arquivo.    | `pnpm test:e2e -- dxf-constructs`                                                                            |
| X5  | A leitura DEVE aceitar fim de linha `LF` e `CRLF`.                                                                                   | `cargo test --manifest-path kernel/Cargo.toml -p neocad-io dxf::pairs`                                       |
| X6  | SE o arquivo está truncado ou com par código/valor inválido, ENTÃO a leitura DEVE devolver erro com a linha do problema, sem pânico. | `cargo test --manifest-path kernel/Cargo.toml -p neocad-io dxf::`                                            |
| X7  | A gravação DEVE produzir `$ACADVER = AC1032`, e o arquivo gravado DEVE abrir sem erro em leitor de referência independente.          | teste novo em `neocad-io` + validação com leitor de código aberto de referência (a definir no ticket)        |
| X8  | A leitura DEVE aceitar arquivos de `AC1009` (R12) a `AC1032`.                                                                        | **falta** corpus de origem pública por versão (achado G4 do review)                                          |

## Questões em aberto

- Leitor de referência para X7 e corpus público para X8: escolher no ticket, conferindo a
  licença (T10).
