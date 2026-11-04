<!-- Caminho relativo: docs/specs/kp4-blocos.md -->

# KP4 — Blocos e atributos

- **Estado:** Esqueleto — critérios EARS a fechar quando a fase abrir
- **Fase:** KP · **ADRs:** 0007 · **Transversais e relacionadas:** 00, 02

## Problema e resultado esperado

Símbolos normalizados, carimbos e listas de materiais vivem em blocos com atributos. O kernel lê definições de bloco, mas a referência de bloco (`INSERT`) é contada como perda.

## Escopo

Referência de bloco com escala, rotação e matriz retangular; atributos (definição, valor, edição); criação de bloco a partir da seleção; inserção; explosão; gravação de bloco em arquivo; eliminação de definições não usadas.

## Fora de escopo

Blocos dinâmicos (`BEDIT`, parâmetros, estados de visibilidade). Referências externas (`XREF`): decidir se entram em KP4 ou em spec própria.

## Comandos envolvidos

Grafia oficial de cada idioma, de [`08-comandos-autocad.tsv`](08-comandos-autocad.tsv).

| comando EN  | comando PT-BR      | apelido |
| ----------- | ------------------ | ------- |
| `BLOCK`     | `BLOCO`            | `B`     |
| `INSERT`    | `INSERIR`          | `I`     |
| `WBLOCK`    | `GBLOCO`           | `W`     |
| `EXPLODE`   | `EXPLODIR`         | `X`     |
| `ATTDEF`    | `DEFATRIB`         | `ATT`   |
| `ATTEDIT`   | `EDITARATRIB`      | `ATE`   |
| `ATTIPEDIT` | `EDITATRIBNOLOCAL` | `ATI`   |
| `PURGE`     | `ELIMINAR`         | `PU`    |

## Restrições já tomadas

- Toda entidade nova é lida e gravada em DXF no mesmo ticket (00, T5).
- Todo comando aceita os dois idiomas e o apelido (spec 08).

## Questões em aberto

- Referências externas (`XR`, `XA`): engenharia as usa muito em pranchas compostas; decidir o lugar.
