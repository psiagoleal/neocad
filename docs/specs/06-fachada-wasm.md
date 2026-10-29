<!-- Caminho relativo: docs/specs/06-fachada-wasm.md -->

# 06 — Fachada WebAssembly

- **Estado:** Aprovada
- **Fase:** K3, bloco D (MT-K3-12) · **ADRs:** 0001, 0003 · **Transversais:** 00

## Problema e resultado esperado

As consultas e operações de K3 só existem para o teste até chegarem à interface. A
fachada `CadSession` passa a oferecer ao frontend as consultas da spec 04 e as operações
da spec 05, com desfazer e refazer atravessando a fronteira, pela única porta permitida
(`src/lib/services/cad-document.ts`, ADR 0001).

## Escopo

Funções da fachada para as consultas G1–G6 da spec 04 (interseção, região, acerto,
prendimento) e as operações da spec 05 (as dez da tabela); tipos NeoCAD correspondentes em `src/lib/types/cad.ts`; conversão
validada no serviço.

## Fora de escopo

Qualquer elemento de interface: botão, cursor, linha de comando (spec 08) e desenho na
tela (spec 07).

## Critérios de aceite (EARS)

| id  | critério                                                                                                                                                                                                                                                                                                             | prova                                                                                 |
| --- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| W1  | Cada consulta e operação do escopo DEVE estar disponível no serviço `cad-document`, com tipos NeoCAD.                                                                                                                                                                                                                | `pnpm test -- cad-document` + `pnpm check`                                            |
| W1b | Nenhum arquivo de `src/` além de `src/lib/services/cad-document.ts` e do teste dele DEVE importar de `src/lib/kernel/pkg`.                                                                                                                                                                                           | regra `no-restricted-imports` no ESLint (`pnpm lint`) (a criar)                       |
| W2  | QUANDO uma operação é executada pelo serviço, desfazer e refazer pelo serviço DEVEM restaurar e reaplicar o resultado; QUANDO não há o que desfazer ou refazer, a chamada DEVE ser inócua e informar isso pelo retorno, sem erro.                                                                                    | `pnpm test -- cad-document`                                                           |
| W3  | SE o kernel recusa a operação, ENTÃO o serviço DEVE devolver erro tipado com a mensagem do kernel, e o documento NÃO DEVE mudar.                                                                                                                                                                                     | `pnpm test -- cad-document`                                                           |
| W4  | SE a resposta da fachada não tem a forma declarada em `src/lib/types/cad.ts`, ENTÃO a conversão DEVE lançar o erro tipado de resposta inválida, nomeando o campo, e não produzir dado parcial.                                                                                                                       | `pnpm test -- cad-document`                                                           |
| W4b | SE o WebAssembly aborta durante uma chamada (pânico ou _trap_), ENTÃO o serviço DEVE lançar erro tipado de falha interna e marcar o documento como não confiável; toda chamada seguinte sobre ele DEVE falhar com o mesmo erro, exceto a gravação de cópia. O que a interface oferece ao usuário é da fase de UI/UX. | `pnpm test -- cad-document` com fachada simulada que lança `WebAssembly.RuntimeError` |
| W5  | O `.wasm` de release DEVE ter no máximo 465 000 bytes (≈ 303 KiB medidos em 2026-10-29 + 50%); o limite só sobe por ADR.                                                                                                                                                                                             | `pnpm kernel:build` + verificação de tamanho no script de build (a criar)             |
