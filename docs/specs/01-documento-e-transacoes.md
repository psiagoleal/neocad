<!-- Caminho relativo: docs/specs/01-documento-e-transacoes.md -->

# 01 — Documento e transações

- **Estado:** Vigente
- **Fase:** K1 (concluída) · **ADRs:** 0003 · **Transversais:** 00 (T1, T2)

## Problema e resultado esperado

Um CAD sem desfazer confiável não serve para trabalho real, e o upstream não oferece
desfazer. O NeoCAD tem modelo de documento próprio, em Rust, em que **toda mutação passa
por transação reversível**, e `Desfazer`/`Refazer` funcionam na interface.

## Escopo

Arena de entidades com identificadores estáveis; tabelas de camadas, blocos e estilos de
texto; documento que as agrega; journal de mudanças; pilha de comandos com desfazer e
refazer; menu `Editar` da interface.

## Fora de escopo

Geometria e operações de edição (specs 04 e 05); leitura e gravação de arquivo (02).

## Critérios de aceite (EARS)

| id  | critério                                                                                                                                       | prova                                                                              |
| --- | ---------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| D1  | O documento DEVE conter sempre a camada `0` e o espaço-modelo, que não podem ser removidos.                                                    | `cargo test --manifest-path kernel/Cargo.toml -p neocad-model -- layer:: block::`  |
| D2  | QUANDO uma entidade é removida, o identificador dela NÃO DEVE voltar a designar outra entidade.                                                | `cargo test --manifest-path kernel/Cargo.toml -p neocad-model arena::`             |
| D3  | O documento DEVE recusar nome de camada vazio ou duplicado, sem diferenciar maiúsculas de minúsculas.                                          | `cargo test --manifest-path kernel/Cargo.toml -p neocad-model layer::`             |
| D4  | QUANDO uma transação é confirmada, a pilha DEVE permitir desfazê-la e refazê-la, e uma nova transação DEVE descartar o que havia para refazer. | `cargo test --manifest-path kernel/Cargo.toml -p neocad-transaction stack::`       |
| D5  | SE uma operação falha no meio de uma transação, ENTÃO o documento DEVE voltar ao estado anterior à transação.                                  | `cargo test --manifest-path kernel/Cargo.toml -p neocad-transaction transaction::` |
| D6  | QUANDO um desenho é aberto, o menu `Editar` DEVE mostrar `Desfazer` e `Refazer` desabilitados.                                                 | `pnpm test:e2e -- kernel-document`                                                 |

## Casos de erro

Identificador inexistente ou já removido devolve erro tipado (`DocumentError`,
`ChangeError`), nunca pânico.
